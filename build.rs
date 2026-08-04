extern crate napi_build;

fn main() {
	#[cfg(not(target_os = "android"))]
	napi_build::setup();
}
