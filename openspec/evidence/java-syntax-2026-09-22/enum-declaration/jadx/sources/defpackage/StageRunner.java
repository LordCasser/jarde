package defpackage;

import java.lang.reflect.Field;

/* JADX INFO: loaded from: StageRunner.class */
class StageRunner {
    StageRunner() {
    }

    static void check(boolean z, String str) {
        if (!z) {
            throw new AssertionError(str);
        }
    }

    public static void main(String[] strArr) throws Exception {
        Stage[] stageArrValues = Stage.values();
        check(stageArrValues.length == 2, "values length");
        check(stageArrValues[0] == Stage.START && stageArrValues[1] == Stage.FINISH, "values declaration order");
        check(Stage.valueOf("START") == Stage.START, "valueOf START");
        check(Stage.valueOf("FINISH") == Stage.FINISH, "valueOf FINISH");
        check(Stage.START.ordinalValue == 4 && Stage.START.code() == 4, "START constructor field");
        check(Stage.FINISH.ordinalValue == 9 && Stage.FINISH.code() == 9, "FINISH constructor field");
        Field field = Stage.class.getField("START");
        Field field2 = Stage.class.getField("FINISH");
        check(field.isEnumConstant() && field2.isEnumConstant(), "constant fields");
        check(field.get(null) == stageArrValues[0] && field2.get(null) == stageArrValues[1], "constant field values");
        System.out.println("enum declaration checks: PASS");
    }
}
