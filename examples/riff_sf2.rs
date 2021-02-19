use riff::{Chunk, ChunkId};

#[derive(Debug)]
struct Root {
    info: SF2Chunk<Vec<SF2Chunk<()>>>,
}

// #[derive(Debug)]
// struct Info {
//     id: ChunkId,
//     ty: ChunkId,
//     children: Vec<(ChunkId, ChunkId, Chunk)>,
// }

#[derive(Debug)]
struct SF2Chunk<DATA> {
    id: ChunkId,
    ty: ChunkId,
    data: DATA,
}

// static IFILL_ID: ChunkId = ChunkId {
//     value: [105, 102, 105, 108],
// };

fn main() {
    let mut file = std::fs::File::open("./testdata/Boomwhacker.sf2").unwrap();

    let chunk = riff::Chunk::read(&mut file, 0).unwrap();

    // println!("{:?}", chunk);

    // root
    let chunks: Vec<_> = chunk.iter(&mut file).collect();

    let info = {
        let info = &chunks[0];
        let id = info.id();
        let ty = info.read_type(&mut file).unwrap();

        let children: Vec<Chunk> = info.iter(&mut file).collect();
        // println!("{:#?}", children);

        let data: Vec<SF2Chunk<()>> = children
            .into_iter()
            .map(|ch| {
                let id = ch.id();
                let ty = ch.read_type(&mut file).unwrap();

                match id.as_str() {
                    "ifil" => {
                        let data = ch.read_contents(&mut file).unwrap();
                        assert_eq!(data.len(), 4);
                        let major = [data[0], data[1]];
                        let minor = [data[2], data[3]];

                        let major: u16 = unsafe { std::mem::transmute::<[u8; 2], u16>(major) };
                        let minor: u16 = unsafe { std::mem::transmute::<[u8; 2], u16>(minor) };

                        println!("{:?}.{:?}", major, minor);
                    }
                    "INAM" => {
                        let data = ch.read_contents(&mut file).unwrap();

                        let name = unsafe { std::ffi::CStr::from_ptr(data.as_ptr() as _) };
                        let name = name.to_str().unwrap();
                        let name = name.to_owned();

                        println!("{:?}", name);
                    }
                    _ => {}
                }

                SF2Chunk { id, ty, data: () }
            })
            .collect();

        // println!("{:#?}, {:#?}", ty, content);

        SF2Chunk { id, ty, data }
    };

    let root = Root { info };

    println!("{:#?}", root);

    // for chunk in chunks.iter() {
    //     let ch_type = chunk.read_type(&mut file).unwrap();
    //     println!("{:#?}", ch_type);
    // }

    // let info = &chunks[0];

    // let info_vec: Vec<_> = info.iter(&mut file).collect();
    // println!("{:#?}", info_vec);
    // let name_raw = info_vec[2].read_contents(&mut file).unwrap();

    // let name = unsafe { std::ffi::CStr::from_ptr(name_raw.as_ptr() as _) };
    // let name = name.to_str().unwrap();
    // let name = name.to_owned();

    // println!("{:#?}", name);
}
