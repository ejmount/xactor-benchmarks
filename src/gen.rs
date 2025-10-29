use crate::Spec;

// Generate the benchmark specifications
pub fn gen_tests(max: Option<u32>) -> Vec<Spec> {
    let max = max.unwrap_or(num_cpus::get() as u32 + 1);
    let mut v = Vec::new();
    for procs in 1..max + 1 {
        for msgs in 1..max + 1 {
            for parallel in 0..(max + 1) {
                //for size in 1..(max + 1) {
                v.push(Spec {
                    procs: 4_u32.pow(procs),
                    messages: 5_u32.pow(msgs),
                    parallel: 2_u32.pow(parallel),
                    size: 10_u32.pow(2),
                })
                //}
            }
        }
    }

    v
}
