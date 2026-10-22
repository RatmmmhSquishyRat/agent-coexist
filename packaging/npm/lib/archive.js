'use strict';

// Extraction for the two archive shapes the release workflow publishes, with
// no dependencies: a gzipped tar for Unix targets and a zip for Windows.
// Both hold the executable at the archive root beside LICENSE and README.md.
// Only the one named entry is extracted; anything else in the archive is
// skipped without being written.

const zlib = require('node:zlib');

/// A failure the launcher expects and reports as one CLI error line: an
/// unpublished platform, a failed download, a checksum mismatch, or a release
/// archive that does not hold the executable. Anything else is a bug and is
/// left to propagate with its stack.
class LauncherError extends Error {
  constructor(message) {
    super(message);
    this.name = 'LauncherError';
  }
}

const TAR_BLOCK = 512;

/// Read a NUL-terminated field as text.
function field(buf, start, len) {
  const slice = buf.subarray(start, start + len);
  const end = slice.indexOf(0);
  return slice.subarray(0, end === -1 ? slice.length : end).toString('latin1');
}

/// The bytes of `name` from a gzipped tar, or null when the archive has no
/// such entry. Octal size fields are read as tar writes them.
function fromTarGz(archive, name) {
  const tar = zlib.gunzipSync(archive);
  for (let at = 0; at + TAR_BLOCK <= tar.length; ) {
    const header = tar.subarray(at, at + TAR_BLOCK);
    if (header.every((b) => b === 0)) return null; // end-of-archive marker
    const entry = field(header, 0, 100);
    const size = parseInt(field(header, 124, 12).trim(), 8);
    if (!Number.isInteger(size)) {
      throw new LauncherError(`tar entry '${entry}' has an unreadable size field`);
    }
    const body = at + TAR_BLOCK;
    if (entry === name) return tar.subarray(body, body + size);
    at = body + Math.ceil(size / TAR_BLOCK) * TAR_BLOCK;
  }
  return null;
}

const EOCD_SIG = 0x06054b50;
const CENTRAL_SIG = 0x02014b50;
const LOCAL_SIG = 0x04034b50;

/// The bytes of `name` from a zip, or null when the archive has no such
/// entry. Stored (method 0) and deflated (method 8) entries are supported;
/// the release workflow produces deflated ones.
function fromZip(archive, name) {
  let eocd = -1;
  for (let at = archive.length - 22; at >= 0; at--) {
    if (archive.readUInt32LE(at) === EOCD_SIG) {
      eocd = at;
      break;
    }
  }
  if (eocd === -1) {
    throw new LauncherError('zip has no end-of-central-directory record');
  }

  const count = archive.readUInt16LE(eocd + 10);
  let at = archive.readUInt32LE(eocd + 16);

  for (let i = 0; i < count; i++) {
    if (archive.readUInt32LE(at) !== CENTRAL_SIG) {
      throw new LauncherError(`zip central directory entry ${i} has a bad signature`);
    }
    const method = archive.readUInt16LE(at + 10);
    const compressed = archive.readUInt32LE(at + 20);
    const nameLen = archive.readUInt16LE(at + 28);
    const extraLen = archive.readUInt16LE(at + 30);
    const commentLen = archive.readUInt16LE(at + 32);
    const localAt = archive.readUInt32LE(at + 42);
    const entry = archive.subarray(at + 46, at + 46 + nameLen).toString('utf8');

    if (entry === name) {
      if (archive.readUInt32LE(localAt) !== LOCAL_SIG) {
        throw new LauncherError(`zip entry '${name}' has a bad local header signature`);
      }
      const localNameLen = archive.readUInt16LE(localAt + 26);
      const localExtraLen = archive.readUInt16LE(localAt + 28);
      const from = localAt + 30 + localNameLen + localExtraLen;
      const body = archive.subarray(from, from + compressed);
      if (method === 0) return body;
      if (method === 8) return zlib.inflateRawSync(body);
      throw new LauncherError(
        `zip entry '${name}' uses unsupported compression method ${method}`,
      );
    }
    at += 46 + nameLen + extraLen + commentLen;
  }
  return null;
}

/// Extract the single named entry from either archive shape, chosen by the
/// archive's own file name.
function extract(archive, archiveName, entryName) {
  const bytes = archiveName.endsWith('.zip')
    ? fromZip(archive, entryName)
    : fromTarGz(archive, entryName);
  if (bytes === null) {
    throw new LauncherError(`${archiveName} does not contain '${entryName}'`);
  }
  return bytes;
}

module.exports = { extract, fromTarGz, fromZip, LauncherError };
