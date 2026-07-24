// EPUB metadata lives in the OPF file, which is referenced from
// META-INF/container.xml. We use roxmltree to navigate the XML

use roxmltree::Document;
use std::io::Read;
use std::path::Path;

use crate::domain::metadata::Metadata;

pub fn extract_metadata(path: &Path) -> Option<Metadata> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let opf_path = find_opf_path(&mut archive)?;
    let opf_content = read_file(&mut archive, &opf_path)?;
    let opf = String::from_utf8_lossy(&opf_content);
    let doc = Document::parse(&opf).ok()?;

    let title = doc
        .descendants()
        .find(|n| n.tag_name().name() == "title")
        .and_then(|n| n.text())
        .map(|s| clean(s.to_string()));

    let author = doc
        .descendants()
        .find(|n| n.tag_name().name() == "creator")
        .and_then(|n| n.text())
        .map(|s| clean(s.to_string()));

    Some(Metadata { title, author })
}

fn find_opf_path(archive: &mut zip::ZipArchive<std::fs::File>) -> Option<String> {
    let content = read_file(archive, "META-INF/container.xml")?;
    let xml = String::from_utf8_lossy(&content);
    let doc = Document::parse(&xml).ok()?;

    doc.descendants()
        .find(|n| n.tag_name().name() == "rootfile")
        .and_then(|n| n.attribute("full-path"))
        .map(|s| s.to_string())
}

fn read_file(archive: &mut zip::ZipArchive<std::fs::File>, path: &str) -> Option<Vec<u8>> {
    let mut file = archive.by_name(path).ok()?;
    let mut buf = Vec::with_capacity(file.size() as usize);
    file.read_to_end(&mut buf).ok()?;
    Some(buf)
}

fn clean(text: String) -> String {
    text.trim_matches('\u{feff}')
        .trim_matches('\0')
        .trim()
        .to_string()
}
