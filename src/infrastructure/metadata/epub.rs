use std::io::Read;
use std::path::Path;

use crate::domain::metadata::Metadata;

// EPUB metadata lives in the OPF file (referenced from META-INF/container.xml).
// We extract <dc:title> and <dc:creator> with simple string matching
// no need for a full XML parser.
pub fn extract_metadata(path: &Path) -> Option<Metadata> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let opf_path = find_opf_path(&mut archive)?;
    let opf_content = read_file(&mut archive, &opf_path)?;
    let opf = String::from_utf8_lossy(&opf_content);

    let title = extract_tag(&opf, "dc:title").map(clean);
    let author = extract_tag(&opf, "dc:creator").map(clean);

    Some(Metadata { title, author })
}

fn find_opf_path(archive: &mut zip::ZipArchive<std::fs::File>) -> Option<String> {
    let content = read_file(archive, "META-INF/container.xml")?;
    let xml = String::from_utf8_lossy(&content);

    let needle = "full-path=\"";
    let start = xml.find(needle)?;
    let rest = &xml[start + needle.len()..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn read_file(archive: &mut zip::ZipArchive<std::fs::File>, path: &str) -> Option<Vec<u8>> {
    let mut file = archive.by_name(path).ok()?;
    let mut buf = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut buf).ok()?;
    Some(buf)
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);

    let start = xml.find(&open)?;
    let rest = &xml[start..];

    let value_start = rest.find('>')? + 1;
    let close = format!("</{}>", tag);
    let value_end = rest[value_start..].find(&close)?;

    let value = rest[value_start..value_start + value_end].trim();
    if value.is_empty() {
        return None;
    }

    Some(value.to_string())
}

fn clean(text: String) -> String {
    text.trim_matches('\u{feff}')
        .trim_matches('\0')
        .trim()
        .to_string()
}
