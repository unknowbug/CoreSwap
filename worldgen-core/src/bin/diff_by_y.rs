// diff_by_y.rs — 挖 dfreg vs actual：读 vanilla .density + Rust finalDensity，按世界 y 层统计 match%，定位差异分布。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;
use std::collections::BTreeMap;

fn be32(b:&[u8], i:&mut usize)->i32{let v=i32::from_be_bytes(b[*i..*i+4].try_into().unwrap());*i+=4;v}
fn be64(b:&[u8], i:&mut usize)->i64{let v=i64::from_be_bytes(b[*i..*i+8].try_into().unwrap());*i+=8;v}

fn main(){
    let seed:i64=-2032795982907864146;
    let mut db=DensityBuilder::new(seed as u64,-64,384);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let df_dir="E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    db.set_external_loader(Box::new(move|_f:&str,name:&str|->String{let p=PathBuf::from(format!("{}\\{}.json",df_dir,name));fs::read_to_string(&p).unwrap_or_else(|e|panic!("\n[LOADFAIL] {}",p.display()))}));
    let settings=parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let fd=settings.get("noise_router").and_then(|r|r.get("final_density")).unwrap();
    let tree:Arc<DensityFunction>=Arc::new(db.build_node(fd).unwrap());
    let b=fs::read("E:\\PYTHON\\CoreSwap\\.investigations\\rust-density-builder\\vanilla_-2032795982907864146_4.density").unwrap();
    let mut i=0usize; let magic=be32(&b,&mut i); let vseed=be64(&b,&mut i); let size=be32(&b,&mut i); let xz=be32(&b,&mut i); let y=be32(&b,&mut i);
    println!("magic=0x{:X} seed={} size={} xzInt={} yInt={}",magic,vseed,size,xz,y);
    let mut by_y:BTreeMap<i32,(u64,u64,u64)> = BTreeMap::new(); // y -> (total, match, maxdiff count>1e-6)
    let mut worst=(0i32,0i32,0i32,0.0f64,0.0f64);
    for _c in 0..(size*size){
        let wx=be32(&b,&mut i); let wz=be32(&b,&mut i);
        let sx=be32(&b,&mut i); let sy=be32(&b,&mut i); let sz=be32(&b,&mut i);
        let min_y=be32(&b,&mut i); let height=be32(&b,&mut i);
        for yi in 0..sy { for zi in 0..sz { for xi in 0..sx {
            let rv=f64::from_bits(be64(&b,&mut i) as u64);
            let x=wx*16+xi*xz; let yy=min_y+yi*y; let zz=wz*16+zi*xz;
            let got=tree.sample(&NoisePos{x,y:yy,z:zz});
            let e=by_y.entry(yy).or_insert((0,0,0));
            e.0+=1; if (got-rv).abs()<1e-9 {e.1+=1;} else {e.2+=1;}
            let d=(got-rv).abs(); if d>worst.3 {worst=(x,yy,zz,rv,got);}
        }}}
    }
    println!("by Y layer: y | total | match% | big-diff(>1e-6) count");
    for (y,(t,m,big)) in by_y.iter() {
        if *t==0 {continue;}
        println!("  y={:4} total={} match={:.1}% big={}", y, *t, 100.0*(*m as f64)/(*t as f64), big);
    }
    println!("worst @({},{},{}) vanilla={} rust={}",worst.0,worst.1,worst.2,worst.3,worst.4);
}
