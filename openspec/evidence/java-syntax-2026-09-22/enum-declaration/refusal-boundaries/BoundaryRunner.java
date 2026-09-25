class BoundaryRunner {
  static void show(Measure value) {
    System.out.println(value.name() + "|" + value.ordinal() + "|" + value.units);
  }

  static void lookup(String name) {
    try {
      Measure value = Measure.valueOf(name);
      System.out.println("valueOf(" + name + ")=" + value.name() + "|" + value.ordinal());
    } catch (IllegalArgumentException error) {
      System.out.println("valueOf(" + name + ")=IllegalArgumentException");
    }
  }

  public static void main(String[] args) {
    show(Measure.LOW);
    show(Measure.HIGH);
    lookup("LOW");
    lookup("HIGH");
    lookup("ALIAS");
    System.out.println("sum=" + Measure.sumUnits() + ", total=" + Measure.totalUnits);
  }
}
