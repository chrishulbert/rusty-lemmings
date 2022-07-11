// This file contains enough code to write a very naive PNG format without needing
// a massive tree of dependencies.

// Converts into an RFC1951 'raw deflate stream' format in a simple non-compressing way.
// An uncompressed deflate stream looks like N blocks, each being:
// [is_final, 2 bytes length, 2 bytes length 1's complement, data].
// Motivation: Avoid dependencies, simple implementation, meet requirements of PNG storage.
// See: https://datatracker.ietf.org/doc/html/rfc1951
// Test output with: ruby -rzlib -e 'print Zlib::Inflate.new(-15).inflate(STDIN.read)' < foo.deflateStream
// https://yob.id.au/2020/06/16/zlib-gzip-and-deflate-in-ruby.html
fn to_deflate_stream(input: &[u8]) -> Vec<u8> {
    if input.is_empty() {
        return vec![1, 0, 0, 0xff, 0xff]; // 1 block with no content.
    }
    let mut output = Vec::<u8>::new();
    let chunks = input.chunks(0xffff);
    let final_index = chunks.len() - 1;
    for (index, chunk) in chunks.enumerate() {
        let is_final = index == final_index;
        output.push(if is_final { 1 } else { 0 });
        let len = chunk.len();
        let len_lsb = (len & 0xff) as u8;
        let len_msb = (len >> 8) as u8;
        output.push(len_lsb); // Max len.
        output.push(len_msb);
        output.push(!len_lsb); // 1's complement of len.
        output.push(!len_msb);
        output.extend_from_slice(chunk);
    }
    output
}

// Converts to an uncompressed RFC1950 zlib stream.
// See: https://datatracker.ietf.org/doc/html/rfc1950
// In the simple uncompressed case, this boils down to adding a header and checksum.
// Test output with: ruby -rzlib -e 'print Zlib::Inflate.new.inflate(STDIN.read)' < foo.zlib
fn to_zlib_stream(input: &[u8]) -> Vec<u8> {
    // Header.
    let mut output = Vec::<u8>::new();
    output.push(0x78); // CMF byte. Bits 0-3=method, 4-7=info/window size. Method=8, Window size=7.
    output.push(1); // FLG byte. Bits 0-4=fcheck, 5=fdict which we dont want so 0, 6-7=flevel where 0 means fastest.

    // Body.
    let deflated = to_deflate_stream(input);
    output.extend(deflated);

    // Checksum.
    // See: https://en.wikipedia.org/wiki/Adler-32#Example_implementation
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for data in input {
        a = (a + (*data as u32)) % 65521;
        b = (b + a) % 65521;
    }
    output.push((b >> 8) as u8);
    output.push((b & 0xff) as u8);
    output.push((a >> 8) as u8);
    output.push((a & 0xff) as u8);

    output
}

// http://libpng.org/pub/png/spec/1.0/PNG-CRCAppendix.html
fn crc(data: &[u8]) -> u32 {
    // Make the CRC table first.
    let mut crc_table: [u32; 256] = [0; 256];
    for n in 0..256 {
        let mut c: u32 = n as u32;
        for _k in 0..8 {
            if c & 1 == 1 {
                c = 0xedb88320u32 ^ (c >> 1);
            } else {
                c = c >> 1;
            }
        }
        crc_table[n] = c;
    }

    // Calculate the CRC.
    let mut crc: u32 = 0xffffffff;
    for b in data {
        crc = crc_table[((crc ^ (*b as u32)) & 0xff) as usize] ^ (crc >> 8);        
    }
    !crc
}

// Append a u32 to a vec, msb first.
fn append_msb(vec: &mut Vec<u8>, value: u32) {
    vec.push((value >> 24) as u8);
    vec.push(((value >> 16) & 0xff) as u8);
    vec.push(((value >> 8) & 0xff) as u8);
    vec.push((value & 0xff) as u8);
}

// https://en.wikipedia.org/wiki/Portable_Network_Graphics#File_format
pub fn png_data(width: u32, height: u32, image_data: &[u32]) -> Vec<u8> {
    let mut output = Vec::<u8>::new();

    // Header.
    output.push(0x89);
    output.push(b'P');
    output.push(b'N');
    output.push(b'G');
    output.push(0x0d); // Cr
    output.push(0x0a); // Lf
    output.push(0x1a); // Eof
    output.push(0x0a); // Lf

    // Build IHDR.
    let mut ihdr_type_and_data = Vec::<u8>::new();
    ihdr_type_and_data.push(b'I');
    ihdr_type_and_data.push(b'H');
    ihdr_type_and_data.push(b'D');
    ihdr_type_and_data.push(b'R');
    append_msb(&mut ihdr_type_and_data, width);
    append_msb(&mut ihdr_type_and_data, height);
    ihdr_type_and_data.push(8); // 8bpp.
    ihdr_type_and_data.push(6); // RGBA.
    ihdr_type_and_data.push(0); // Compression method: zlib.
    ihdr_type_and_data.push(0); // Filter method.
    ihdr_type_and_data.push(0); // No interlace.
    let ihdr_len = ihdr_type_and_data.len() - 4; // Minus the type.
    let ihdr_crc = crc(&ihdr_type_and_data);
    // Append IHDR to output.
    append_msb(&mut output, ihdr_len as u32);
    output.extend_from_slice(&ihdr_type_and_data);
    append_msb(&mut output, ihdr_crc);

    // Build image data.
    // Left-right, then Top-bottom.
    // Each line is prepended a filter type byte (0).
    let mut idat_data = Vec::<u8>::new();
    let mut data_iter = image_data.iter();
    for _y in 0..height {
        idat_data.push(0); // Filter.
        for _x in 0..width {
            append_msb(&mut idat_data, *data_iter.next().unwrap());
        }
    }
    let compressed_idat_data = to_zlib_stream(&idat_data);

    // Build IDAT.
    let mut idat_type_and_data = Vec::<u8>::new();
    idat_type_and_data.push(b'I');
    idat_type_and_data.push(b'D');
    idat_type_and_data.push(b'A');
    idat_type_and_data.push(b'T');
    idat_type_and_data.extend_from_slice(&compressed_idat_data);
    let idat_len = idat_type_and_data.len() - 4; // Minus the type.
    let idat_crc = crc(&idat_type_and_data);
    // Append IDAT to output.
    append_msb(&mut output, idat_len as u32);
    output.extend_from_slice(&idat_type_and_data);
    append_msb(&mut output, idat_crc);

    // IEND (no data).
    append_msb(&mut output, 0); // Length.
    output.push(b'I'); // Type.
    output.push(b'E');
    output.push(b'N');
    output.push(b'D');
    let iend_crc = crc(b"IEND");
    append_msb(&mut output, iend_crc);

    output
}
