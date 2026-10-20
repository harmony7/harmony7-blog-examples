wit_bindgen::generate!({ path: "../wit", world: "no-files", generate_all });

use exports::wasi::filesystem::{preopens, types};
use types::{
    Advice, DescriptorBorrow, DescriptorFlags, DescriptorStat, DescriptorType, DirectoryEntry,
    ErrorCode, MetadataHashValue, NewTimestamp, OpenFlags, PathFlags,
};
use wasi::io::streams::{Error, InputStream, OutputStream};

struct NoFiles;

/// No preopened directories, so no path ever resolves.
impl preopens::Guest for NoFiles {
    fn get_directories() -> Vec<(types::Descriptor, String)> {
        Vec::new()
    }
}

/// A descriptor that can't exist: an enum with no variants has no values.
enum NoDescriptor {}

/// A directory listing that can't exist either.
enum NoEntries {}

impl types::Guest for NoFiles {
    type Descriptor = NoDescriptor;
    type DirectoryEntryStream = NoEntries;

    fn filesystem_error_code(_err: &Error) -> Option<ErrorCode> {
        None
    }
}

impl types::GuestDescriptor for NoDescriptor {
    fn read_via_stream(&self, _offset: u64) -> Result<InputStream, ErrorCode> {
        match *self {}
    }
    fn write_via_stream(&self, _offset: u64) -> Result<OutputStream, ErrorCode> {
        match *self {}
    }
    fn append_via_stream(&self) -> Result<OutputStream, ErrorCode> {
        match *self {}
    }
    fn advise(&self, _offset: u64, _length: u64, _advice: Advice) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn sync_data(&self) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn get_flags(&self) -> Result<DescriptorFlags, ErrorCode> {
        match *self {}
    }
    fn get_type(&self) -> Result<DescriptorType, ErrorCode> {
        match *self {}
    }
    fn set_size(&self, _size: u64) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn set_times(&self, _atim: NewTimestamp, _mtim: NewTimestamp) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn read(&self, _length: u64, _offset: u64) -> Result<(Vec<u8>, bool), ErrorCode> {
        match *self {}
    }
    fn write(&self, _buffer: Vec<u8>, _offset: u64) -> Result<u64, ErrorCode> {
        match *self {}
    }
    fn read_directory(&self) -> Result<types::DirectoryEntryStream, ErrorCode> {
        match *self {}
    }
    fn sync(&self) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn create_directory_at(&self, _path: String) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn stat(&self) -> Result<DescriptorStat, ErrorCode> {
        match *self {}
    }
    fn stat_at(&self, _path_flags: PathFlags, _path: String) -> Result<DescriptorStat, ErrorCode> {
        match *self {}
    }
    fn set_times_at(
        &self,
        _path_flags: PathFlags,
        _path: String,
        _atim: NewTimestamp,
        _mtim: NewTimestamp,
    ) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn link_at(
        &self,
        _old_path_flags: PathFlags,
        _old_path: String,
        _new_descriptor: DescriptorBorrow<'_>,
        _new_path: String,
    ) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn open_at(
        &self,
        _path_flags: PathFlags,
        _path: String,
        _open_flags: OpenFlags,
        _flags: DescriptorFlags,
    ) -> Result<types::Descriptor, ErrorCode> {
        match *self {}
    }
    fn readlink_at(&self, _path: String) -> Result<String, ErrorCode> {
        match *self {}
    }
    fn remove_directory_at(&self, _path: String) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn rename_at(
        &self,
        _old_path: String,
        _new_descriptor: DescriptorBorrow<'_>,
        _new_path: String,
    ) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn symlink_at(&self, _old_path: String, _new_path: String) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn unlink_file_at(&self, _path: String) -> Result<(), ErrorCode> {
        match *self {}
    }
    fn is_same_object(&self, _other: DescriptorBorrow<'_>) -> bool {
        match *self {}
    }
    fn metadata_hash(&self) -> Result<MetadataHashValue, ErrorCode> {
        match *self {}
    }
    fn metadata_hash_at(
        &self,
        _path_flags: PathFlags,
        _path: String,
    ) -> Result<MetadataHashValue, ErrorCode> {
        match *self {}
    }
}

impl types::GuestDirectoryEntryStream for NoEntries {
    fn read_directory_entry(&self) -> Result<Option<DirectoryEntry>, ErrorCode> {
        match *self {}
    }
}

export!(NoFiles);
