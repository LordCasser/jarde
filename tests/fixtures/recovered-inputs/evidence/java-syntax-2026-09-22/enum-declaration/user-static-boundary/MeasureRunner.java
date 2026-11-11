class MeasureRunner {
  static void check(boolean condition, String message) {
    if (!condition) throw new AssertionError(message);
  }

  public static void main(String[] args) {
    Measure[] values = Measure.values();
    check(values.length == 2, "values count");
    check(values[0] == Measure.LOW && values[1] == Measure.HIGH, "values order");
    check(Measure.valueOf("LOW") == Measure.LOW, "valueOf");
    check(Measure.LOW.units == 2 && Measure.HIGH.units == 5, "constructor values");
    check(Measure.totalUnits == 7 && Measure.sumUnits() == 7, "user static initialization/method");
    System.out.println("user static boundary: PASS");
  }
}
