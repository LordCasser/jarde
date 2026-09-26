class TwrReturnTail {
  static java.io.ByteArrayInputStream open() {
    return new java.io.ByteArrayInputStream(new byte[0]);
  }
  static int run(int input) throws Exception {
    try (java.io.ByteArrayInputStream resource = open()) {
      int value = resource.read();
      return value;
    }
  }
  static int runSaved(int input) throws Exception {
    try (java.io.ByteArrayInputStream resource = open()) {
      input = resource.read();
      return input;
    }
  }
  static int effect() throws Exception {
    int value;
    try (java.io.ByteArrayInputStream resource = open()) {
      value = resource.read();
    }
    value++;
    return value;
  }
}
