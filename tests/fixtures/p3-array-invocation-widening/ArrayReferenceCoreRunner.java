public class ArrayReferenceCoreRunner {
    private interface Call {
        String run();
    }

    private static String error(Throwable error) {
        return error.getClass().getName();
    }

    private static void call(String name, Call value) {
        try {
            System.out.println(name + "=" + value.run());
        } catch (Throwable error) {
            System.out.println(name + "=error:" + error(error));
        }
    }

    public static void main(String[] args) {
        final int[][] intMatrix = {new int[0], new int[3]};
        final String[] strings = new String[2];
        final String[][] stringMatrix = {new String[0], new String[4]};
        final int[][] emptyInt = new int[0][];
        final String[] emptyStrings = new String[0];
        final String[][] emptyStringMatrix = new String[0][];
        final int[][] intNull = null;
        final String[] stringNull = null;
        final String[][] stringMatrixNull = null;

        call("target:int[][]", () -> ArrayReferenceCore.typedIntMatrix(intMatrix));
        call("target:String[]", () -> ArrayReferenceCore.typedStringArray(strings));
        call("target:String[][]", () -> ArrayReferenceCore.typedStringMatrix(stringMatrix));
        call("target:Object-widened-int[][]", () -> ArrayReferenceCore.widenedObject(intMatrix));
        call("target:Object[]-widened-int[][]", () -> ArrayReferenceCore.widenedObjectArray(intMatrix));
        call("target:Object-widened-String[]", () -> ArrayReferenceCore.widenedObject(strings));
        call("target:Object[]-widened-String[]", () -> ArrayReferenceCore.widenedObjectArray(strings));
        call("target:Object-widened-String[][]", () -> ArrayReferenceCore.widenedObject(stringMatrix));
        call("target:Object[]-widened-String[][]", () -> ArrayReferenceCore.widenedObjectArray(stringMatrix));

        call("exact:int[][]", () -> ArrayReferenceCore.exactIntMatrix(intMatrix));
        call("exact:String[]", () -> ArrayReferenceCore.exactStringArray(strings));
        call("exact:String[][]", () -> ArrayReferenceCore.exactStringMatrix(stringMatrix));
        call("exact:Object[]-int[][]", () -> ArrayReferenceCore.exactObjectArray(intMatrix));
        call("exact:Object[]-String[]", () -> ArrayReferenceCore.exactObjectArray(strings));
        call("exact:Object[]-String[][]", () -> ArrayReferenceCore.exactObjectArray(stringMatrix));
        call("exact:Object[][]-String[][]", () -> ArrayReferenceCore.exactObjectMatrix(stringMatrix));
        call("describe:Object[]-int[][]", () -> ArrayReferenceCore.describeObjectArray(intMatrix));
        call("describe:Object[]-String[]", () -> ArrayReferenceCore.describeObjectArray(strings));
        call("describe:Object[][]-String[][]", () -> ArrayReferenceCore.describeObjectMatrix(stringMatrix));

        call("size:empty-int[][]", () -> ArrayReferenceCore.exactIntMatrix(emptyInt));
        call("size:empty-String[]", () -> ArrayReferenceCore.exactStringArray(emptyStrings));
        call("size:empty-String[][]", () -> ArrayReferenceCore.exactStringMatrix(emptyStringMatrix));

        call("null:exact-int[][]", () -> ArrayReferenceCore.exactIntMatrix(intNull));
        call("null:exact-String[]", () -> ArrayReferenceCore.exactStringArray(stringNull));
        call("null:exact-String[][]", () -> ArrayReferenceCore.exactStringMatrix(stringMatrixNull));
        call("null:Object-widened", () -> ArrayReferenceCore.widenedObject(intNull));
        call("null:Object[]-widened", () -> ArrayReferenceCore.widenedObjectArray(stringMatrixNull));
        call("null:Object[]-dereference", () -> ArrayReferenceCore.describeObjectArray(stringNull));
    }
}
