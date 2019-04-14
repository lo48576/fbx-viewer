# fbx-viewer

[![Build Status](https://travis-ci.org/lo48576/fbx-viewer.svg?branch=develop)](https://travis-ci.org/lo48576/fbx-viewer)


## Usage

### Prerequisite

* Vulkan
    + This viewer requires Vulkan to be available on the computer.
    + About Vulkan, see this page: [Vulkan Overview - The Khronos Group Inc](https://www.khronos.org/vulkan/).
* OS
    + The developer tested the viewer on Linux machine.
    + The viewer is not tested on Mac and Windows.
      Feel free to report if some problems happen on your platform.

### Recommended resources

* [Nakano Sisters](https://web.archive.org/web/20180805180846/http://nakasis.com/)
    + [FBX files distributed here](https://web.archive.org/web/20180805180846/http://nakasis.com/data/NakanoSisters_1_2_FBX.zip)
      is used as test file.
    + The viewer works well (maybe not quite well) with these FBX files.

### Run the viewer

Run the command below:

```
$ cargo run -- PATH_TO_FBX_FILE.fbx
```

For who want to debug:

```
$ RUST_LOG=fbx_viewer=trace RUST_BACKTRACE=1 VK_INSTANCE_LAYERS=VK_LAYER_LUNARG_standard_validation cargo run -- PATH_TO_FBX_FILE.fbx
```

### Move the camera

* Move
    + `0`: Reset the camera position.
    + `W`: Move tha camera forward.
    + `A`: Move tha camera left.
    + `S`: Move tha camera backward.
    + `D`: Move tha camera right.
    + `Shift-W`: Move tha camera upward.
    + `Shift-S`: Move tha camera downward.
* Rotate
    + `Ctrl-0`: Reset the camera angle.
    + `Ctrl-W`: Rotate the camera up.
    + `Ctrl-A`: Rotate the camera left.
    + `Ctrl-S`: Rotate the camera down.
    + `Ctrl-D`: Rotate the camera right.


## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE.txt](LICENSE-APACHE.txt) or
  <https://www.apache.org/licenses/LICENSE-2.0>)
* MIT license ([LICENSE-MIT.txt](LICENSE-MIT.txt) or
  <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
