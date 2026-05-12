// Copyright 2025 MongoDB, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0

use rust_rocksdb::{Options, ReadOptions, SstFileReader, SstFileWriter};

#[test]
fn test_sst_file_reader_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let sst_path = dir.path().join("test.sst");

    // Write an SST file
    let opts = Options::default();
    let mut writer = SstFileWriter::create(&opts);
    writer.open(&sst_path).unwrap();
    writer.put(b"key1", b"value1").unwrap();
    writer.put(b"key2", b"value2").unwrap();
    writer.put(b"key3", b"value3").unwrap();
    writer.put(b"key4", b"value4").unwrap();
    writer.put(b"key5", b"value5").unwrap();
    writer.finish().unwrap();

    // Read it back with SstFileReader
    let reader = SstFileReader::create(&opts);
    reader.open(&sst_path).unwrap();
    reader.verify_checksum().unwrap();

    let readopts = ReadOptions::default();
    let mut iter = reader.new_iterator(&readopts);

    // Iterate from the beginning
    iter.seek_to_first();
    assert!(iter.valid());
    assert_eq!(iter.key(), Some(b"key1".as_ref()));
    assert_eq!(iter.value(), Some(b"value1".as_ref()));

    iter.next();
    assert!(iter.valid());
    assert_eq!(iter.key(), Some(b"key2".as_ref()));
    assert_eq!(iter.value(), Some(b"value2".as_ref()));

    // Seek to a specific key
    iter.seek(b"key4");
    assert!(iter.valid());
    assert_eq!(iter.key(), Some(b"key4".as_ref()));
    assert_eq!(iter.value(), Some(b"value4".as_ref()));

    // Seek past the end
    iter.seek(b"key9");
    assert!(!iter.valid());

    // Seek for prev
    iter.seek_for_prev(b"key35");
    assert!(iter.valid());
    assert_eq!(iter.key(), Some(b"key3".as_ref()));

    // Iterate from the end
    iter.seek_to_last();
    assert!(iter.valid());
    assert_eq!(iter.key(), Some(b"key5".as_ref()));
    assert_eq!(iter.value(), Some(b"value5".as_ref()));

    iter.prev();
    assert!(iter.valid());
    assert_eq!(iter.key(), Some(b"key4".as_ref()));

    // Count all entries
    iter.seek_to_first();
    let mut count = 0;
    while iter.valid() {
        count += 1;
        iter.next();
    }
    assert_eq!(count, 5);
    iter.status().unwrap();
}

#[test]
fn test_sst_file_reader_empty_iteration() {
    let dir = tempfile::tempdir().unwrap();
    let sst_path = dir.path().join("single.sst");

    let opts = Options::default();
    let mut writer = SstFileWriter::create(&opts);
    writer.open(&sst_path).unwrap();
    writer.put(b"only_key", b"only_value").unwrap();
    writer.finish().unwrap();

    let reader = SstFileReader::create(&opts);
    reader.open(&sst_path).unwrap();

    let readopts = ReadOptions::default();
    let mut iter = reader.new_iterator(&readopts);

    // Seek to non-existent key past the end
    iter.seek(b"zzz");
    assert!(!iter.valid());
    assert_eq!(iter.key(), None);
    assert_eq!(iter.value(), None);
}

#[test]
fn test_sst_file_reader_large_dataset() {
    let dir = tempfile::tempdir().unwrap();
    let sst_path = dir.path().join("large.sst");

    let opts = Options::default();
    let mut writer = SstFileWriter::create(&opts);
    writer.open(&sst_path).unwrap();

    let n = 10_000;
    for i in 0..n {
        let key = format!("k{:08}", i);
        let val = format!("v{:08}", i);
        writer.put(key.as_bytes(), val.as_bytes()).unwrap();
    }
    writer.finish().unwrap();

    let reader = SstFileReader::create(&opts);
    reader.open(&sst_path).unwrap();
    reader.verify_checksum().unwrap();

    let readopts = ReadOptions::default();
    let mut iter = reader.new_iterator(&readopts);

    // Point lookup in the middle
    iter.seek(format!("k{:08}", 5000).as_bytes());
    assert!(iter.valid());
    assert_eq!(iter.key(), Some(format!("k{:08}", 5000).as_bytes()));
    assert_eq!(iter.value(), Some(format!("v{:08}", 5000).as_bytes()));

    // Count all
    iter.seek_to_first();
    let mut count = 0;
    while iter.valid() {
        count += 1;
        iter.next();
    }
    assert_eq!(count, n);
}
