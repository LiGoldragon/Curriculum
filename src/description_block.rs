//! YAML 1.2.2 section 8.1, for a description at the frontmatter root.
//! https://yaml.org/spec/1.2.2/#81-block-scalar-styles
use std::iter::Peekable;

enum BlockStyle {
    Literal,
    Folded,
}

enum Chomping {
    Strip,
    Clip,
    Keep,
}

struct BlockHeader {
    style: BlockStyle,
    chomping: Chomping,
    indentation: Option<usize>,
}

impl BlockHeader {
    fn parse(header: &str) -> Result<Self, String> {
        let mut characters = header.trim().chars().peekable();
        let style = match characters.next() {
            Some('|') => BlockStyle::Literal,
            Some('>') => BlockStyle::Folded,
            _ => return Err("invalid description block style".into()),
        };
        let mut indentation = None;
        let mut chomping = None;
        while let Some(character) = characters.next() {
            match character {
                '1'..='9' if indentation.is_none() => {
                    indentation = character.to_digit(10).map(|value| value as usize);
                }
                '-' if chomping.is_none() => chomping = Some(Chomping::Strip),
                '+' if chomping.is_none() => chomping = Some(Chomping::Keep),
                ' ' | '\t' => {
                    let suffix: String = characters.collect();
                    if !suffix.trim().is_empty() && !suffix.trim_start().starts_with('#') {
                        return Err("invalid description block header suffix".into());
                    }
                    break;
                }
                _ => return Err("invalid description block header".into()),
            }
        }
        Ok(Self {
            style,
            chomping: chomping.unwrap_or(Chomping::Clip),
            indentation,
        })
    }
}

fn indentation(line: &str) -> usize {
    line.bytes().take_while(|byte| *byte == b' ').count()
}

pub(crate) fn decode<'a>(
    header: &str,
    lines: &mut Peekable<impl Iterator<Item = &'a str>>,
) -> Result<String, String> {
    let header = BlockHeader::parse(header)?;
    let mut content = Vec::new();
    let mut level = header.indentation;
    while let Some(&line) = lines.peek() {
        let spaces = indentation(line);
        let empty = line.bytes().all(|byte| byte == b' ');
        if !empty && spaces == 0 {
            break;
        }
        if !empty {
            let level = *level.get_or_insert(spaces);
            if spaces < level {
                if line.trim_start().starts_with('#') {
                    break;
                }
                return Err("insufficient description block indentation".into());
            }
        }
        content.push(line);
        lines.next();
    }
    let level = level.unwrap_or_else(|| content.iter().map(|line| line.len()).max().unwrap_or(0));
    let mut seen_content = false;
    let content = content
        .into_iter()
        .map(|line| {
            let spaces = indentation(line);
            let empty = spaces == line.len();
            if header.indentation.is_none() && !seen_content && empty && spaces > level {
                return Err("overindented leading description block line".to_owned());
            }
            seen_content |= !empty;
            Ok(&line[spaces.min(level)..])
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut output = String::new();
    let mut index = 0;
    while index < content.len() {
        let line = content[index];
        output.push_str(line);
        if matches!(header.style, BlockStyle::Literal) || line.is_empty() {
            output.push('\n');
            index += 1;
            continue;
        }
        let next = (index + 1..content.len()).find(|&next| !content[next].is_empty());
        match next {
            Some(next) => {
                let blanks = next - index - 1;
                if line.starts_with([' ', '\t']) || content[next].starts_with([' ', '\t']) {
                    output.extend(std::iter::repeat_n('\n', blanks + 1));
                } else if blanks == 0 {
                    output.push(' ');
                } else {
                    output.extend(std::iter::repeat_n('\n', blanks));
                }
                index = next;
            }
            None => {
                output.extend(std::iter::repeat_n('\n', content.len() - index));
                break;
            }
        }
    }
    match header.chomping {
        Chomping::Keep => (),
        Chomping::Strip => output.truncate(output.trim_end_matches('\n').len()),
        Chomping::Clip => {
            output.truncate(output.trim_end_matches('\n').len());
            if !output.is_empty() {
                output.push('\n');
            }
        }
    }
    Ok(output)
}
