// density_builder.rs — worldgen JSON → DensityFunction 树（DensityBuilder：seed 派生 + noise registry + registry/lazyRef）
// 对齐 C++ density_builder.h：DensityBuilder(seed, noiseParams) -> randomDeriver = XoroshiroRandom(seed).nextSplitter()
//                    getNoiseSampler(key) = randomDeriver.split(key) -> DoublePerlinNoiseSampler(rnd, noiseParams[key])
//                    buildNode(type 分派) / buildSpline(SplineData) / resolveRef(registry 懒引用)
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use crate::density::*;
use crate::json::JsonValue;
use crate::noise::DoublePerlinNoiseSampler;
use crate::noise::NoiseParameters;
use crate::noise::OctavePerlinNoiseSampler;
use crate::xoroshiro::XoroshiroRandom;
use crate::xoroshiro::XoroshiroSplitter;

// buildNoiseParams（C++ density_probe.cpp buildNoiseParams：BuiltinNoiseParameters 1.20.1 全表；noise_params.json 读取后续）
pub fn build_noise_params() -> HashMap<String, NoiseParameters> {
    let mut m: HashMap<String, NoiseParameters> = HashMap::new();
    let mut add = |key: &str, oct: i32, amps: Vec<f64>| {
        m.insert(format!("minecraft:{}", key), NoiseParameters { first_octave: oct, amplitudes: amps });
    };
    add("temperature", -10, vec![1.5, 0.0, 1.0, 0.0, 0.0, 0.0]);
    add("vegetation", -8, vec![1.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
    add("continentalness", -9, vec![1.0, 1.0, 2.0, 2.0, 2.0, 1.0, 1.0, 1.0, 1.0]);
    add("erosion", -9, vec![1.0, 1.0, 0.0, 1.0, 1.0]);
    add("ridge", -7, vec![1.0, 2.0, 1.0, 0.0, 0.0, 0.0]);
    add("offset", -3, vec![1.0, 1.0, 1.0, 0.0]);
    add("aquifer_barrier", -3, vec![1.0]);
    add("aquifer_fluid_level_floodedness", -7, vec![1.0]);
    add("aquifer_lava", -1, vec![1.0]);
    add("aquifer_fluid_level_spread", -5, vec![1.0]);
    add("pillar", -7, vec![1.0, 1.0]);
    add("pillar_rareness", -8, vec![1.0]);
    add("pillar_thickness", -8, vec![1.0]);
    add("spaghetti_2d", -7, vec![1.0]);
    add("spaghetti_2d_elevation", -8, vec![1.0]);
    add("spaghetti_2d_modulator", -11, vec![1.0]);
    add("spaghetti_2d_thickness", -11, vec![1.0]);
    add("spaghetti_3d_1", -7, vec![1.0]);
    add("spaghetti_3d_2", -7, vec![1.0]);
    add("spaghetti_3d_rarity", -11, vec![1.0]);
    add("spaghetti_3d_thickness", -8, vec![1.0]);
    add("spaghetti_roughness", -5, vec![1.0]);
    add("spaghetti_roughness_modulator", -8, vec![1.0]);
    add("cave_entrance", -7, vec![0.4, 0.5, 1.0]);
    add("cave_layer", -8, vec![1.0]);
    add("cave_cheese", -8, vec![0.5, 1.0, 2.0, 1.0, 2.0, 1.0, 0.0, 2.0, 0.0]);
    add("ore_veininess", -8, vec![1.0]);
    add("ore_vein_a", -7, vec![1.0]);
    add("ore_vein_b", -7, vec![1.0]);
    add("ore_gap", -5, vec![1.0]);
    add("noodle", -8, vec![1.0]);
    add("noodle_thickness", -8, vec![1.0]);
    add("noodle_ridge_a", -7, vec![1.0]);
    add("noodle_ridge_b", -7, vec![1.0]);
    add("jagged", -16, vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    add("surface", -6, vec![1.0, 1.0, 1.0]);
    add("surface_secondary", -6, vec![1.0, 1.0, 0.0, 1.0]);
    add("clay_bands_offset", -8, vec![1.0]);
    add("badlands_pillar", -2, vec![1.0, 1.0, 1.0, 1.0]);
    add("badlands_pillar_roof", -8, vec![1.0]);
    add("badlands_surface", -6, vec![1.0, 1.0, 1.0]);
    add("iceberg_pillar", -6, vec![1.0, 1.0, 1.0, 1.0]);
    add("iceberg_pillar_roof", -3, vec![1.0]);
    add("iceberg_surface", -6, vec![1.0, 1.0, 1.0]);
    add("surface_swamp", -2, vec![1.0]);
    add("calcite", -9, vec![1.0, 1.0, 1.0, 1.0]);
    add("gravel", -8, vec![1.0, 1.0, 1.0, 1.0]);
    add("powder_snow", -6, vec![1.0, 1.0, 1.0, 1.0]);
    add("packed_ice", -7, vec![1.0, 1.0, 1.0, 1.0]);
    add("ice", -4, vec![1.0, 1.0, 1.0, 1.0]);
    add("soul_sand_layer", -8, vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334]);
    add("gravel_layer", -8, vec![1.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334]);
    add("patch", -5, vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.013333333333333334]);
    add("netherrack", -3, vec![1.0, 0.0, 0.0, 0.35]);
    add("nether_wart", -3, vec![1.0, 0.0, 0.0, 0.9]);
    add("nether_state_selector", -4, vec![1.0]);
    m
}

// 从 `noise_params.json`（权威 BuiltinNoiseParameters 1.20.1 导出）加载噪声参数表——对齐基准从硬编码表切到文件（judge P2-e）。
// 格式：{"minecraft:<key>": {"firstOctave": <int>, "amplitudes": [<f64>,...]}, ...}
pub fn build_noise_params_from_file(path: &str) -> Result<HashMap<String, NoiseParameters>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("read {}: {}", path, e))?;
    let v = crate::json::parse(&text).map_err(|e| format!("parse {}: {}", path, e))?;
    let mut m = HashMap::new();
    if let JsonValue::Object(entries) = v {
        for (k, v) in entries {
            let oct = v.get("firstOctave").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
            let mut amps = Vec::new();
            if let Some(a) = v.get("amplitudes") {
                if let JsonValue::Array(arr) = a {
                    for x in arr { amps.push(x.as_f64().unwrap_or(1.0)); }
                }
            }
            m.insert(k.clone(), NoiseParameters { first_octave: oct, amplitudes: amps });
        }
    } else {
        return Err("noise_params.json root not object".into());
    }
    Ok(m)
}

// DensityBuilder：seed -> randomDeriver(Splitter)，noise sampler 懒建（split(key)），registry（lazyRef 懒引用）
pub struct DensityBuilder {
    #[allow(dead_code)] seed: u64,
    noise_params: HashMap<String, NoiseParameters>,
    random_deriver: XoroshiroSplitter,
    noise_samplers: HashMap<String, Arc<DoublePerlinNoiseSampler>>,
    registry: HashMap<String, Arc<DensityFunction>>,
    lazy_refs: HashMap<String, Arc<Mutex<Option<Arc<DensityFunction>>>>>,
    // 惰性加载器：(fullRef, shortName) -> JSON 文本（外部设置，用于按需加载 registry 引用）
    external_loader: Option<Box<dyn Fn(&str, &str) -> String>>,
    #[allow(dead_code)] min_y: i32,
    #[allow(dead_code)] noise_height: i32,
}

impl DensityBuilder {
    pub fn new(seed: u64, min_y: i32, noise_height: i32) -> Self {
        let noise_params = build_noise_params();
        let mut base = XoroshiroRandom::new(seed);
        let random_deriver = base.next_splitter();
        DensityBuilder { seed, noise_params, random_deriver, noise_samplers: HashMap::new(), registry: HashMap::new(), lazy_refs: HashMap::new(), external_loader: None, min_y, noise_height }
    }
    pub fn set_external_loader(&mut self, loader: Box<dyn Fn(&str, &str) -> String>) {
        self.external_loader = Some(loader);
    }
    // 用 noise_params.json 覆盖硬编码表（对齐基准切到权威文件，judge P2-e）。须在首次 get_noise_sampler 前调用。
    pub fn load_noise_params_file(&mut self, path: &str) -> Result<(), String> {
        self.noise_params = build_noise_params_from_file(path)?;
        Ok(())
    }
    pub fn get_noise_sampler(&mut self, key: &str) -> Arc<DoublePerlinNoiseSampler> {
        if let Some(s) = self.noise_samplers.get(key) { return s.clone(); }
        let params = self.noise_params.get(key).cloned().ok_or_else(|| "unknown noise params".to_string()).expect("noise params");
        let mut rnd = self.random_deriver.split_str(key);
        let sampler = Arc::new(crate::noise::DoublePerlinNoiseSampler::new(&mut rnd, &params));
        self.noise_samplers.insert(key.to_string(), sampler.clone());
        sampler
    }
    // getNoiseSamplerFromObj（C++ L335-339）：obj.noise .str() -> getNoiseSampler
    fn get_noise_sampler_from_obj(&mut self, obj: &JsonValue) -> Arc<DoublePerlinNoiseSampler> {
        let n = obj.get("noise").ok_or("noise field missing").expect("noise field missing");
        self.get_noise_sampler(&n.as_str().unwrap_or("").to_string())
    }
    pub fn register_function(&mut self, key: &str, df: Arc<DensityFunction>) {
        // 若已有 lazyRef/占位（循环引用），填充其 target（对齐 C++ registerFunction L285-297）
        if let Some(old) = self.registry.get(key) {
            if let DensityFunction::Lazy { target } = &**old {
                *target.lock().unwrap() = Some(df.clone());
            }
        }
        self.registry.insert(key.to_string(), df.clone());
        for (k, lr) in &self.lazy_refs {
            if k == key && lr.lock().unwrap().is_none() {
                *lr.lock().unwrap() = Some(df.clone());
            }
        }
    }
    pub fn new_lazy_ref(&mut self, key: &str) {
        self.lazy_refs.insert(key.to_string(), Arc::new(Mutex::new(None)));
    }
    // 解析 registry 引用（"minecraft:overworld/xxx" 等），对齐 C++ resolveRef L221-263
    pub fn resolve_ref(&mut self, key: &str) -> Arc<DensityFunction> {
        if let Some(s) = self.registry.get(key) { return s.clone(); }
        if key == "minecraft:shift_x" {
            let ns = self.get_noise_sampler("minecraft:offset");
            let df = DensityFunction::Wrapping { input: Box::new(DensityFunction::ShiftDF { noise: ns, mode: ShiftMode::ShiftA }) };
            let rc = Arc::new(df);
            self.registry.insert(key.to_string(), rc.clone());
            return rc;
        }
        if key == "minecraft:shift_z" {
            let ns = self.get_noise_sampler("minecraft:offset");
            let df = DensityFunction::Wrapping { input: Box::new(DensityFunction::ShiftDF { noise: ns, mode: ShiftMode::ShiftB }) };
            let rc = Arc::new(df);
            self.registry.insert(key.to_string(), rc.clone());
            return rc;
        }
        if key == "minecraft:y" {
            // y = yClampedGradient(minY, maxY, minY, maxY)（恒等 y 映射，overworld -64..320）
            let df = DensityFunction::YClampedGradient { from_y: -64, to_y: 320, from_value: -64.0, to_value: 320.0 };
            let rc = Arc::new(df);
            self.registry.insert(key.to_string(), rc.clone());
            return rc;
        }
        if key == "minecraft:zero" {
            let rc = Arc::new(DensityFunction::Constant { value: 0.0 });
            self.registry.insert(key.to_string(), rc.clone());
            return rc;
        }
        // 惰性按需加载：minecraft:overworld/<name>
        if key.starts_with("minecraft:overworld/") && self.external_loader.is_some() {
            let name = key["minecraft:overworld/".len()..].to_string();
            // 循环引用保护：先注册 LazyRef 占位，加载期间若再引用 ref 会命中占位（对齐 C++ L252-253）
            let placeholder = Arc::new(DensityFunction::Lazy { target: Arc::new(Mutex::new(None)) });
            self.registry.insert(key.to_string(), placeholder.clone());
            let json_text = (self.external_loader.as_ref().unwrap())(key, &name);
            if !json_text.trim().is_empty() {
                let root = crate::json::parse(&json_text).map_err(|_| key.to_string()).unwrap_or(JsonValue::Null);
                let df = self.build_node(&root).unwrap_or_else(|e| panic!("resolve {} failed: {}", key, e));
                if let DensityFunction::Lazy { target } = &*placeholder {
                    *target.lock().unwrap() = Some(Arc::new(df.clone()));
                }
                let rc = Arc::new(df);
                self.registry.insert(key.to_string(), rc.clone());
                return rc;
            }
            self.registry.remove(key);
        }
        panic!("unresolved density function ref: {}", key);
    }
    fn bin(&self, op: BinOp, a: DensityFunction, b: DensityFunction) -> DensityFunction {
        let d = a.min_value(); let e = b.min_value();
        let f = a.max_value(); let g = b.max_value();
        let (h, i) = match op {
            BinOp::Add => (d + e, f + g),
            BinOp::Max => (d.max(e), f.max(g)),
            BinOp::Min => (d.min(e), f.min(g)),
            BinOp::Mul => {
                let hh = if d > 0.0 && e > 0.0 { d * e } else if f < 0.0 && g < 0.0 { f * g } else { (d * g).min(f * e) };
                let ii = if d > 0.0 && e > 0.0 { f * g } else if f < 0.0 && g < 0.0 { d * e } else { (d * e).max(f * g) };
                (hh, ii)
            }
        };
        // 常量折叠：add/mul 带 Constant → LinearOperation（sample 等价：x*const 或 x+const）——对齐 C++ create L111-119
        if op == BinOp::Add || op == BinOp::Mul {
            if let DensityFunction::Constant { value } = a {
                return DensityFunction::LinearOp { op, input: Box::new(b), c: value, mn: h, mx: i };
            }
            if let DensityFunction::Constant { value } = b {
                return DensityFunction::LinearOp { op, input: Box::new(a), c: value, mn: h, mx: i };
            }
        }
        DensityFunction::BinaryOp { op, a: Box::new(a), b: Box::new(b), mn: h, mx: i }
    }
    fn un(&self, op: UnaryOp, input: DensityFunction) -> DensityFunction {
        let imin = input.min_value(); let imax = input.max_value();
        let mut mn = apply_unary(op, imin);
        let mut mx = apply_unary(op, imax);
        // 对齐 C++ UnaryOperation::create L184-188：ABS/SQUARE 的 mn 用 max(0.0, imin)（raw imin，非 |imin|）
        if op == UnaryOp::Abs || op == UnaryOp::Square {
            mn = imin.max(0.0);
            mx = apply_unary(op, imin).max(apply_unary(op, imax));
        }
        if mn > mx { std::mem::swap(&mut mn, &mut mx); }
        DensityFunction::UnaryOp { op, input: Box::new(input), mn, mx }
    }
    pub fn build_node(&mut self, v: &JsonValue) -> Result<DensityFunction, String> {
        // 数字/字符串裸节点（对齐 C++ buildNode L31-47）：数字=Constant，字符串=引用
        match v {
            JsonValue::Number(n) => return Ok(DensityFunction::Constant { value: *n }),
            JsonValue::String(s) => return Ok(self.resolve_ref(s).as_ref().clone()),
            _ => {}
        }
        let t = v.get("type").and_then(|x| x.as_str()).unwrap_or("");
        // 引用（registry 查找）
        if t.starts_with("minecraft:overworld/") || t.starts_with("minecraft:nether/") {
            return Ok(self.resolve_ref(t).as_ref().clone());
        }
        Ok(match t {
            "minecraft:constant" => DensityFunction::Constant { value: v.get("value").and_then(|x| x.as_f64()).unwrap_or(0.0) },
            "minecraft:add" => { let a = self.build_node(self.arg(v, "argument1"))?; let b = self.build_node(self.arg(v, "argument2"))?; self.bin(BinOp::Add, a, b) }
            "minecraft:mul" => { let a = self.build_node(self.arg(v, "argument1"))?; let b = self.build_node(self.arg(v, "argument2"))?; self.bin(BinOp::Mul, a, b) }
            "minecraft:min" => { let a = self.build_node(self.arg(v, "argument1"))?; let b = self.build_node(self.arg(v, "argument2"))?; self.bin(BinOp::Min, a, b) }
            "minecraft:max" => { let a = self.build_node(self.arg(v, "argument1"))?; let b = self.build_node(self.arg(v, "argument2"))?; self.bin(BinOp::Max, a, b) }
            "minecraft:abs" => { let i = self.build_node(self.arg(v, "argument"))?; self.un(UnaryOp::Abs, i) }
            "minecraft:square" => { let i = self.build_node(self.arg(v, "argument"))?; self.un(UnaryOp::Square, i) }
            "minecraft:cube" => { let i = self.build_node(self.arg(v, "argument"))?; self.un(UnaryOp::Cube, i) }
            "minecraft:half_negative" => { let i = self.build_node(self.arg(v, "argument"))?; self.un(UnaryOp::HalfNegative, i) }
            "minecraft:quarter_negative" => { let i = self.build_node(self.arg(v, "argument"))?; self.un(UnaryOp::QuarterNegative, i) }
            "minecraft:squeeze" => { let i = self.build_node(self.arg(v, "argument"))?; self.un(UnaryOp::Squeeze, i) }
            "minecraft:clamp" => { let i = self.build_node(self.arg(v, "input"))?; let mn = v.get("min").and_then(|x| x.as_f64()).unwrap_or(0.0); let mx = v.get("max").and_then(|x| x.as_f64()).unwrap_or(0.0); DensityFunction::Clamp { input: Box::new(i), mn, mx } }
            "minecraft:interpolated" => { let inner = Arc::new(self.build_node(self.arg(v, "argument"))?); let min_y = v.get("min_y").and_then(|x| x.as_f64()).unwrap_or(self.min_y as f64) as i32; let height = v.get("height").and_then(|x| x.as_f64()).unwrap_or(self.noise_height as f64) as i32; DensityFunction::Interpolated(InterpolatedData::new(inner, min_y, height)) }
            // 2D/3D 缓存不做（性能优化），但 cache_all_in_cell/cache_once 纯委托包装（对齐 C++ WrappingDF L644-652）
            "minecraft:flat_cache" => DensityFunction::FlatCache(FlatCacheData::new(Arc::new(self.build_node(self.arg(v, "argument"))?))),
            "minecraft:cache_2d" => DensityFunction::Cache2D(Cache2DData::new(Arc::new(self.build_node(self.arg(v, "argument"))?))),
            "minecraft:cache_once" | "minecraft:cache_all_in_cell" => DensityFunction::Wrapping { input: Box::new(self.build_node(self.arg(v, "argument"))?) },
            "minecraft:spline" => self.build_spline(v)?,
            "minecraft:noise" => {
                let key = v.get("noise").and_then(|x| x.as_str()).unwrap_or("minecraft:continentalness");
                let xz = v.get("xz_scale").and_then(|x| x.as_f64()).unwrap_or(1.0);
                let ys = v.get("y_scale").and_then(|x| x.as_f64()).unwrap_or(1.0);
                let ns = self.get_noise_sampler(key);
                let mx = ns.get_max_value();
                DensityFunction::Noise { noise: ns, xz_scale: xz, y_scale: ys, mn: -mx, mx }
            }
            "minecraft:shifted_noise" => {
                let n = v.get("noise").ok_or("shifted_noise missing noise")?;
                let ns = self.get_noise_sampler(&n.as_str().unwrap_or("").to_string());
                let xz = v.get("xz_scale").and_then(|x| x.as_f64()).unwrap_or(1.0);
                let y = v.get("y_scale").and_then(|x| x.as_f64()).unwrap_or(1.0);
                let sx = self.opt_shift(v, "shift_x")?;
                let sy = self.opt_shift(v, "shift_y")?;
                let sz = self.opt_shift(v, "shift_z")?;
                DensityFunction::ShiftedNoise { shift_x: Box::new(sx), shift_y: Box::new(sy), shift_z: Box::new(sz), xz_scale: xz, y_scale: y, noise: ns }
            }
            "minecraft:shift_a" => { let ns = self.get_noise_sampler_from_obj(v); DensityFunction::ShiftDF { noise: ns, mode: ShiftMode::ShiftA } }
            "minecraft:shift_b" => { let ns = self.get_noise_sampler_from_obj(v); DensityFunction::ShiftDF { noise: ns, mode: ShiftMode::ShiftB } }
            "minecraft:shift" => { let ns = self.get_noise_sampler_from_obj(v); DensityFunction::ShiftDF { noise: ns, mode: ShiftMode::Shift } }
            "minecraft:range_choice" => {
                let input = self.build_node(self.arg(v, "input"))?;
                let mn = v.get("min_inclusive").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let mx = v.get("max_exclusive").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let ir = self.build_node(self.arg(v, "when_in_range"))?;
                let oor = self.build_node(self.arg(v, "when_out_of_range"))?;
                DensityFunction::RangeChoice { input: Box::new(input), min_inclusive: mn, max_exclusive: mx, in_range: Box::new(ir), out_of_range: Box::new(oor) }
            }
            "minecraft:y_clamped_gradient" => {
                let fy = v.get("from_y").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
                let ty = v.get("to_y").and_then(|x| x.as_f64()).unwrap_or(0.0) as i32;
                let fv = v.get("from_value").and_then(|x| x.as_f64()).unwrap_or(0.0);
                let tv = v.get("to_value").and_then(|x| x.as_f64()).unwrap_or(0.0);
                DensityFunction::YClampedGradient { from_y: fy, to_y: ty, from_value: fv, to_value: tv }
            }
            "minecraft:weird_scaled_sampler" => {
                let n = v.get("noise").ok_or("weird_scaled missing noise")?;
                let ns = self.get_noise_sampler(&n.as_str().unwrap_or("").to_string());
                let rv = v.get("rarity_value_mapper").and_then(|x| x.as_str());
                let rarity = if rv == Some("type_2") { WeirdRarity::Caves } else { WeirdRarity::Tunnels };
                let input = self.build_node(self.arg(v, "input"))?;
                // C++ 注意（L142）：rarity_value_mapper "type_2"=CAVES 漏下划线会全误判 TUNNELS（8576 差根因）
                DensityFunction::WeirdScaled { input: Box::new(input), noise: ns, rarity }
            }
            "minecraft:blend_alpha" => DensityFunction::BlendAlpha,
            "minecraft:blend_offset" => DensityFunction::BlendOffset,
            "minecraft:blend_density" => DensityFunction::BlendDensity { input: Box::new(self.build_node(self.arg(v, "argument"))?) },
            "minecraft:old_blended_noise" => {
                let mut rnd = self.random_deriver.split_str("minecraft:terrain");
                let xzs = v.get("xz_scale").and_then(|x| x.as_f64()).unwrap_or(0.25);
                let ys = v.get("y_scale").and_then(|x| x.as_f64()).unwrap_or(0.125);
                let xzf = v.get("xz_factor").and_then(|x| x.as_f64()).unwrap_or(80.0);
                let yf = v.get("y_factor").and_then(|x| x.as_f64()).unwrap_or(160.0);
                let smear = v.get("smear_scale_multiplier").and_then(|x| x.as_f64()).unwrap_or(8.0);
                // C++ InterpolatedNoiseDF 构造顺序：lower → upper → interpolation，均消费 rnd
                let amp_l = OctavePerlinNoiseSampler::range_closed_amplitudes(-15, 0);
                let lower = OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
                let upper = OctavePerlinNoiseSampler::new_legacy(&mut rnd, -15, &amp_l);
                let amp_i = OctavePerlinNoiseSampler::range_closed_amplitudes(-7, 0);
                let interp = OctavePerlinNoiseSampler::new_legacy(&mut rnd, -7, &amp_i);
                DensityFunction::InterpolatedNoise(InterpolatedNoiseData::new(lower, upper, interp, xzs, ys, xzf, yf, smear))
            }
            _ => return Err(format!("unsupported density type '{}' on node {:?}", t, v)),
        })
    }
    // 可选 shift 子节点（shifted_noise 的 shift_x/y/z 缺失 → Constant(0)，对齐 C++ L105-107）
    fn opt_shift(&mut self, v: &JsonValue, key: &str) -> Result<DensityFunction, String> {
        if let Some(cv) = v.get(key) { self.build_node(cv) } else { Ok(DensityFunction::Constant { value: 0.0 }) }
    }
    fn arg<'a>(&self, v: &'a JsonValue, key: &str) -> &'a JsonValue { v.get(key).unwrap_or(&JsonValue::Null) }

    // ---- buildSpline（SplineData 构建——对齐 C++ buildSplineNode：先子节点再本节点）----
    pub fn build_spline(&mut self, obj_in: &JsonValue) -> Result<DensityFunction, String> {
        let obj: &JsonValue = if let JsonValue::Object(_) = obj_in { if let Some(s) = obj_in.get("spline") { s } else { obj_in } } else { obj_in };
        let mut sb = SplineBuilder::new();
        let root = Self::build_spline_node(self, &mut sb, obj)?;
        Ok(DensityFunction::Spline(sb.finish(root)))
    }
    fn build_spline_node(b: &mut DensityBuilder, sb: &mut SplineBuilder, obj: &JsonValue) -> Result<i32, String> {
        let points = obj.get("points").and_then(|x| x.as_array()).ok_or("spline points")?;
        let n = points.len() as i32;
        let mut locs = vec![0f32; n as usize];
        let mut ders = vec![0f32; n as usize];
        let mut child_ids = vec![0i32; n as usize];
        for i in 0..n as usize {
            let p = &points[i];
            locs[i] = p.get("location").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
            ders[i] = p.get("derivative").and_then(|x| x.as_f64()).unwrap_or(0.0) as f32;
            if let Some(value) = p.get("value") {
                if let JsonValue::Number(_) = value {
                    child_ids[i] = sb.add_leaf(value.as_f64().unwrap_or(0.0) as f32);
                } else {
                    child_ids[i] = Self::build_spline_node(b, sb, value)?;
                }
            }
        }
        let coord = obj.get("coordinate").ok_or("spline coordinate")?;
        let loc_fn = Arc::new(b.build_node(coord)?);
        let node_id = sb.add_node(loc_fn, n);
        for i in 0..n as usize { sb.add_point(locs[i], ders[i], child_ids[i]); }
        Ok(node_id)
    }
}

pub struct SplineBuilder { nodes: Vec<SplineNode>, locations: Vec<f32>, derivatives: Vec<f32>, sub_idx: Vec<i32>, loc_fns: Vec<Arc<DensityFunction>> }
impl SplineBuilder {
    fn new() -> Self { SplineBuilder { nodes: Vec::new(), locations: Vec::new(), derivatives: Vec::new(), sub_idx: Vec::new(), loc_fns: Vec::new() } }
    fn add_leaf(&mut self, value: f32) -> i32 { self.nodes.push(SplineNode { loc_fn: -1, loc_begin: 0, sub_begin: 0, n: 0, fixed_value: value }); (self.nodes.len() - 1) as i32 }
    fn add_node(&mut self, loc_fn: Arc<DensityFunction>, n: i32) -> i32 { let lb = self.locations.len() as i32; let sb = self.sub_idx.len() as i32; let lfi = self.loc_fns.len() as i32; self.loc_fns.push(loc_fn); self.nodes.push(SplineNode { loc_fn: lfi, loc_begin: lb, sub_begin: sb, n, fixed_value: 0.0 }); (self.nodes.len() - 1) as i32 }
    fn add_point(&mut self, loc: f32, deriv: f32, child: i32) { self.locations.push(loc); self.derivatives.push(deriv); self.sub_idx.push(child); }
    fn finish(self, root: i32) -> SplineData {
        let mut s = SplineData { nodes: self.nodes, locations: self.locations, derivatives: self.derivatives, sub_idx: self.sub_idx, loc_fns: self.loc_fns, root, min_val: 0.0, max_val: 0.0 };
        // 缓存 min/max（O(1) 查询，避免 BinaryOp 每点递归 node_min/node_max）
        s.min_val = s.node_min(root);
        s.max_val = s.node_max(root);
        s
    }
}

// 便捷：无 registry / seed 的 build_node（顶层 finalDensity 用）
pub fn build_node(v: &JsonValue) -> Result<DensityFunction, String> {
    let mut db = DensityBuilder::new(0, -64, 384);
    db.build_node(v)
}
