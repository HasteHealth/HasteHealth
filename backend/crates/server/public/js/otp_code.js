(function () {
  var code = document.getElementById("otp_code");
  var inputs = Array.prototype.slice.call(
    document.querySelectorAll(".digit_input"),
  );

  function syncCode() {
    code.value = inputs
      .map(function (input) {
        return input.value;
      })
      .join("");
  }

  inputs.forEach(function (input, index) {
    input.addEventListener("input", function () {
      input.value = input.value.replace(/[^0-9]/g, "").slice(-1);
      syncCode();
      if (input.value && index < inputs.length - 1) {
        inputs[index + 1].focus();
      }
    });

    input.addEventListener("keydown", function (event) {
      if (event.key === "Backspace" && !input.value && index > 0) {
        inputs[index - 1].focus();
      }
    });

    input.addEventListener("paste", function (event) {
      event.preventDefault();
      var digits = (event.clipboardData.getData("text") || "")
        .replace(/[^0-9]/g, "")
        .split("");
      digits.forEach(function (digit, offset) {
        if (inputs[index + offset]) {
          inputs[index + offset].value = digit;
        }
      });
      syncCode();
      var next = inputs[Math.min(index + digits.length, inputs.length - 1)];
      next.focus();
    });
  });

  if (inputs[0]) {
    inputs[0].focus();
  }
})();
