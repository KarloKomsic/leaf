// Generates cover thumbnails for PDF and EPUB files and caches them
// as JPEGs in a local cover_cache/ directory. If a cached cover
// already exists, we skip generation entirely.

use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn cached_cover_path(path: &Path) -> Option<PathBuf> {
    let cache_dir = cache_dir();
    fs::create_dir_all(&cache_dir).ok()?;

    let hash = path_hash(path);
    let cached = cache_dir.join(format!("{}.jpg", hash));

    if cached.exists() {
        return Some(cached);
    }

    generate_cover(path, &cached)?;
    Some(cached)
}

fn cache_dir() -> PathBuf {
    PathBuf::from("cover_cache")
}

fn path_hash(path: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn generate_cover(path: &Path, output: &Path) -> Option<()> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();

    match ext.as_str() {
        "pdf" => generate_pdf_cover(path, output),
        "epub" => generate_epub_cover(path, output),
        _ => None,
    }
}

// Uses pdftoppm (from poppler-utils) to render the first page as a
// JPEG thumbnail. We rename pdftoppm's output to a predictable path
// so it can be found on subsequent lookups.
fn generate_pdf_cover(path: &Path, output: &Path) -> Option<()> {
    let prefix = output.with_extension("");

    let result = Command::new("pdftoppm")
        .arg("-jpeg")
        .arg("-r")
        .arg("72")
        .arg("-f")
        .arg("1")
        .arg("-l")
        .arg("1")
        .arg("-scale-to")
        .arg("200")
        .arg(path)
        .arg(&prefix)
        .output()
        .ok()?;

    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        eprintln!("    pdftoppm error for {}: {}", path.display(), stderr.trim());
        return None;
    }

    let generated = format!("{}-1.jpg", prefix.display());
    let generated_path = PathBuf::from(&generated);

    if generated_path.exists() {
        fs::rename(&generated_path, output).ok()
    } else {
        let filename = prefix.file_name()?.to_string_lossy();
        let dir = prefix.parent()?;
        let entries: Vec<_> = fs::read_dir(dir).ok()?.collect();
        for entry in entries {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(filename.as_ref()) && name.ends_with(".jpg") {
                fs::rename(entry.path(), output).ok()?;
                return Some(());
            }
        }
        eprintln!("    pdftoppm output not found for {}", path.display());
        None
    }
}

// Extracts the cover image from an EPUB by reading the OPF manifest
// and finding the file that the metadata points to as the cover.
fn generate_epub_cover(path: &Path, output: &Path) -> Option<()> {
    let file = fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    let (opf_path, opf_bytes) = read_opf(&mut archive)?;
    let opf = String::from_utf8_lossy(&opf_bytes);

    let cover_href = find_cover_href(&opf)?;

    let base = opf_path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let abs_path = if base.is_empty() {
        cover_href
    } else {
        format!("{}/{}", base, cover_href)
    };

    let mut cover_file = archive.by_name(&abs_path).ok()?;
    let mut buf = Vec::with_capacity(cover_file.size() as usize);
    cover_file.read_to_end(&mut buf).ok()?;

    fs::write(output, &buf).ok()
}

// Reads the OPF file path from META-INF/container.xml, then returns
// the OPF content along with its path for resolving relative hrefs.
fn read_opf(archive: &mut zip::ZipArchive<fs::File>) -> Option<(String, Vec<u8>)> {
    let container = {
        let mut f = archive.by_name("META-INF/container.xml").ok()?;
        let mut buf = Vec::with_capacity(f.size() as usize);
        f.read_to_end(&mut buf).ok()?;
        buf
    };

    let xml = String::from_utf8_lossy(&container);
    let needle = "full-path=\"";
    let start = xml.find(needle)?;
    let rest = &xml[start + needle.len()..];
    let end = rest.find('"')?;
    let opf_path = rest[..end].to_string();

    let opf_bytes = {
        let mut f = archive.by_name(&opf_path).ok()?;
        let mut buf = Vec::with_capacity(f.size() as usize);
        f.read_to_end(&mut buf).ok()?;
        buf
    };

    Some((opf_path, opf_bytes))
}

// Scans the OPF XML for a <meta> element with name="cover" and then
// looks up the corresponding <item> element to get the image href.
fn find_cover_href(opf: &str) -> Option<String> {
    let meta_needle = "name=\"cover\" content=\"";
    let meta_needle2 = "name='cover' content='";

    let cover_id = opf
        .find(meta_needle)
        .or_else(|| opf.find(meta_needle2))
        .and_then(|start| {
            let rest = &opf[start..];
            let value_start = rest.find("content=\"")? + "content=\"".len();
            let end = rest[value_start..].find('"')?;
            Some(rest[value_start..value_start + end].to_string())
        })
        .or_else(|| {
            let start = opf.find(meta_needle2)?;
            let rest = &opf[start..];
            let value_start = rest.find("content='")? + "content='".len();
            let end = rest[value_start..].find('\'')?;
            Some(rest[value_start..value_start + end].to_string())
        })?;

    let item_needle = format!("id=\"{}\"", cover_id);
    let item_needle2 = format!("id='{}'", cover_id);

    let item_section = opf
        .find(&item_needle)
        .or_else(|| opf.find(&item_needle2))
        .map(|start| &opf[start..])?;

    let href_needle = "href=\"";
    let alt_href = "href='";

    item_section
        .find(href_needle)
        .or_else(|| item_section.find(alt_href))
        .and_then(|start| {
            let rest = &item_section[start..];
            let value_start = rest.find("href=\"")? + "href=\"".len();
            let end = rest[value_start..].find('"')?;
            Some(rest[value_start..value_start + end].to_string())
        })
        .or_else(|| {
            let rest = &item_section;
            let value_start = rest.find("href='")? + "href='".len();
            let end = rest[value_start..].find('\'')?;
            Some(rest[value_start..value_start + end].to_string())
        })
}
