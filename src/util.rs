use rand::Rng;

pub fn fmt_output_workspace(output: &str, workspace: &str) -> String {
    let mut rng = rand::thread_rng();
    let new_idx = rng.gen::<u8>().to_string();
    format!("{}:{}-{}", output, workspace, new_idx)
}
