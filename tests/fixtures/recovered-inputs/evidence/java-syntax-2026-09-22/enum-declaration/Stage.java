enum Stage {
  START(4), FINISH(9);

  final int ordinalValue;

  Stage(int ordinalValue) {
    this.ordinalValue = ordinalValue;
  }

  int code() {
    return ordinalValue;
  }
}
