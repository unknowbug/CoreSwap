// banddiff.rs — 正确种子(-2032)：逐 Y 层统计 Rust(finalDensity+Aquifer) vs vanilla 块匹配率，
// 定位 surface 层(aquifer/beardifier)缺口在哪几层。输出每层 match%+nonAir match%。
use std::sync::Arc;
use WorldgenRust::density_builder::DensityBuilder;
use WorldgenRust::density::{DensityFunction, NoisePos};
use WorldgenRust::aquifer::Aquifer;
use WorldgenRust::json::parse;
use std::fs;
use std::path::PathBuf;
use std::collections::BTreeMap;

fn be32(b:&[u8], i:&mut usize)->i32{let v=i32::from_be_bytes(b[*i..*i+4].try_into().unwrap());*i+=4;v}
fn be64(b:&[u8], i:&mut usize)->i64{let v=i64::from_be_bytes(b[*i..*i+8].try_into().unwrap());*i+=8;v}
fn be16(b:&[u8], i:&mut usize)->u16{let v=u16::from_be_bytes(b[*i..*i+2].try_into().unwrap());*i+=2;v}

fn main(){
    let seed:i64=-2032795982907864146;
    let mut db=DensityBuilder::new(seed as u64,-64,384);
    db.load_noise_params_file("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\noise_params.json").unwrap();
    let df_dir="E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\density_function\\overworld";
    db.set_external_loader(Box::new(move |_f:&str,name:&str|->String{
        let p=PathBuf::from(format!("{}\\{}.json",df_dir,name));
        fs::read_to_string(&p).unwrap_or_else(|e|panic!("\n[LOADFAIL] {}",p.display()))
    }));
    let settings=parse(&fs::read_to_string("E:\\PYTHON\\CoreSwap\\versions\\1.20.1\\data\\worldgen\\data\\minecraft\\worldgen\\noise_settings\\overworld.json").unwrap()).unwrap();
    let router=settings.get("noise_router").unwrap();
    let tree=Arc::new(db.build_node(router.get("final_density").unwrap()).unwrap());
    let mut b=|k:&str|->Arc<DensityFunction>{Arc::new(db.build_node(router.get(k).unwrap()).unwrap())};
    let barrier=b("barrier"); let flooded=b("fluid_level_floodedness"); let spread=b("fluid_level_spread");
    let lava=b("lava"); let erosion=b("erosion"); let depth=b("depth"); let init=b("initial_density_without_jaggedness");
    let splitter=db.random_deriver().split_str("minecraft:aquifer").next_splitter();

    let bd=fs::read("E:\\PYTHON\\MC\\data\\vanilla_-2032795982907864146_4_0_0.blocks").unwrap();
    let mut i=0usize;
    let _magic=be32(&bd,&mut i); let _seed=be64(&bd,&mut i); let size=be32(&bd,&mut i);
    let _ox=be32(&bd,&mut i); let _oz=be32(&bd,&mut i); let miny=be32(&bd,&mut i); let height=be32(&bd,&mut i);
    let bpc=(16*16*height) as usize;
    let mut per_y:BTreeMap<i32,(u64,u64,u64,u64,u64)> = BTreeMap::new(); // y -> (tot, match, nonair_tot, nonair_match, water_tot)
    for _c in 0..(size*size){
        let cx=be32(&bd,&mut i); let cz=be32(&bd,&mut i);
        let mut vanilla=vec![0i32; bpc];
        for k in 0..bpc { vanilla[k]=be16(&bd,&mut i) as i32; }
        { let mut j=i; for _z in 0..16 { for _x in 0..16 { let ln=be16(&bd,&mut j) as usize; j+=ln; } } i=j; }
        let mut aq=Aquifer::new(barrier.clone(),flooded.clone(),spread.clone(),lava.clone(),erosion.clone(),depth.clone(),init.clone(),splitter.clone(),cx*16,cz*16,miny,height);
        // 块序 i_ = lx + lz*16 + ly*256
        for ly in 0..height {
            let yy=miny+ly;
            for lz in 0..16 { for lx in 0..16 {
                let x=cx*16+lx; let z=cz*16+lz; let y=yy;
                let d=tree.sample(&NoisePos{x,y,z});
                let got=if d>0.0 {1} else { let blk=aq.apply(x,y,z,d); match blk {1=>32,2=>33,_=>0} };
                let v=vanilla[(lx+lz*16+ly*256) as usize];
                let e=per_y.entry(y).or_insert((0,0,0,0,0));
                e.0+=1; if got==v {e.1+=1;}
                if v!=0 { e.2+=1; if got==v {e.3+=1;} }
                if v==32 { e.4+=1; }
            }}
        }
    }
    println!("y | total | match% | nonAir_match/tot | water_tot");
    for (y,(t,m,nt,nm,wt)) in per_y.iter(){
        if *t==0 {continue;}
        // 只打 y in [-64,120]（surface band 附近）+ 大差异
        if *y>-64 && *y<130 {
            println!("y={:4} tot={} m={:.1}%   nonA={}/{} ({:.1}%)  wat={}",
                y,t,100.0*(*m as f64)/(*t as f64),nm,nt,100.0*(*nm as f64)/(*nt as f64),wt);
        }
    }
}
