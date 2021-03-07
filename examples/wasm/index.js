import("./pkg")
  .catch(console.error)
  .then((rust_module) => {
    document.getElementById("start").addEventListener("click", () => {
      let handle = rust_module.beep();

      document.getElementById("C").addEventListener("click", () => {
        rust_module.noteOn(handle, 60);
      });

      document.getElementById("C#").addEventListener("click", () => {
        rust_module.noteOn(handle, 61);
      });

      document.getElementById("D").addEventListener("click", () => {
        rust_module.noteOn(handle, 62);
      });

      document.getElementById("D#").addEventListener("click", () => {
        rust_module.noteOn(handle, 63);
      });

      document.getElementById("E").addEventListener("click", () => {
        rust_module.noteOn(handle, 64);
      });
    });
  });
