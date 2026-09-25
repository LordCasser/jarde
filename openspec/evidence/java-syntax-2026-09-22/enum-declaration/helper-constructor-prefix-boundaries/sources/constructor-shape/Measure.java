enum Measure {
  LOW(2), HIGH(5);

  final int units;
  static int totalUnits;

  Measure(int units) {
    this.units = units;
    BoundarySupport.constructorEffect();
  }

  static {
    totalUnits = sumUnits();
  }

  static int sumUnits() {
    return LOW.units + HIGH.units;
  }
}
