class StageRunner {
  static void check(boolean condition, String message) {
    if (!condition) throw new AssertionError(message);
  }

  public static void main(String[] args) throws Exception {
    Stage[] values = Stage.values();
    check(values.length == 2, "values length");
    check(values[0] == Stage.START && values[1] == Stage.FINISH, "values declaration order");
    check(Stage.valueOf("START") == Stage.START, "valueOf START");
    check(Stage.valueOf("FINISH") == Stage.FINISH, "valueOf FINISH");
    check(Stage.START.ordinalValue == 4 && Stage.START.code() == 4, "START constructor field");
    check(Stage.FINISH.ordinalValue == 9 && Stage.FINISH.code() == 9, "FINISH constructor field");
    java.lang.reflect.Field start = Stage.class.getField("START");
    java.lang.reflect.Field finish = Stage.class.getField("FINISH");
    check(start.isEnumConstant() && finish.isEnumConstant(), "constant fields");
    check(start.get(null) == values[0] && finish.get(null) == values[1], "constant field values");
    System.out.println("enum declaration checks: PASS");
  }
}
