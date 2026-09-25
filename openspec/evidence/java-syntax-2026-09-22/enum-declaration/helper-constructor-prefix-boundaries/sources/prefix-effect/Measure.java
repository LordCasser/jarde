enum Measure {
  LOW(BoundarySupport.prefixValue(2)), HIGH(BoundarySupport.prefixValue(5));

  final int units;
  static int totalUnits;

  Measure(int units) {
    this.units = units;
  }

  static {
    totalUnits = sumUnits();
  }

  static int sumUnits() {
    return LOW.units + HIGH.units;
  }
}
