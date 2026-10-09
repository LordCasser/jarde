package defpackage;
public final class Runner {
    private static void require(boolean condition, String label, String value) {
        if (!condition) {
            throw new AssertionError(label + ":" + value);
        }
        System.out.println("ok:" + label + ":" + value);
    }

    public static void main(String[] args) {
        int[][] two = { { 1, 2 }, { 3, 30 } };
        int result2 = ReturnedIntArrayUpdates.plain2(two, 1, 1, 5);
        if (result2 != 35) throw new AssertionError("returned-value:" + result2);
        require(two[1][1] == 35, "plain2", Integer.toString(two[1][1]));

        int[][][] three = { { { 1, 2 }, { 3, 4 } }, { { 5, 6 }, { 7, 8 } } };
        int result3 = ReturnedIntArrayUpdates.plain3(three, 1, 0, 1, 8);
        if (result3 != 14) throw new AssertionError("returned-value:" + result3);
        require(three[1][0][1] == 14, "plain3", Integer.toString(three[1][0][1]));

        int[] one = { 9 };
        int result1 = ReturnedIntArrayUpdates.scalar(one, 0, 5);
        if (result1 != 14) throw new AssertionError("returned-value:" + result1);
        require(one[0] == 14, "scalar", Integer.toString(one[0]));

        int[][] traced = { { 10 } };
        ReturnedIntArrayUpdates.trace = 0;
        int resultT = ReturnedIntArrayUpdates.traced(traced, 0, 0, 5);
        if (resultT != 15) throw new AssertionError("returned-value:" + resultT);
        require(ReturnedIntArrayUpdates.trace == 123 && traced[0][0] == 15,
                "traced", ReturnedIntArrayUpdates.trace + ":" + traced[0][0]);

        ReturnedIntArrayUpdates.trace = 0;
        boolean outerNull = false;
        try {
            ReturnedIntArrayUpdates.traced(null, 0, 0, 5);
        } catch (NullPointerException expected) {
            outerNull = true;
        }
        require(outerNull && ReturnedIntArrayUpdates.trace == 1,
                "outer-null", Integer.toString(ReturnedIntArrayUpdates.trace));

        ReturnedIntArrayUpdates.trace = 0;
        boolean outerBounds = false;
        try {
            ReturnedIntArrayUpdates.traced(new int[1][1], 2, 0, 5);
        } catch (ArrayIndexOutOfBoundsException expected) {
            outerBounds = true;
        }
        require(outerBounds && ReturnedIntArrayUpdates.trace == 1,
                "outer-oob", Integer.toString(ReturnedIntArrayUpdates.trace));

        ReturnedIntArrayUpdates.trace = 0;
        boolean nullRow = false;
        try {
            ReturnedIntArrayUpdates.traced(new int[][] { null }, 0, 0, 5);
        } catch (NullPointerException expected) {
            nullRow = true;
        }
        require(nullRow && ReturnedIntArrayUpdates.trace == 12,
                "null-row", Integer.toString(ReturnedIntArrayUpdates.trace));

        ReturnedIntArrayUpdates.trace = 0;
        boolean innerBounds = false;
        try {
            ReturnedIntArrayUpdates.traced(new int[][] { { 9 } }, 0, 2, 5);
        } catch (ArrayIndexOutOfBoundsException expected) {
            innerBounds = true;
        }
        require(innerBounds && ReturnedIntArrayUpdates.trace == 12,
                "inner-oob", Integer.toString(ReturnedIntArrayUpdates.trace));

        int[][] replaced = { { 10 } };
        int[] oldRow = replaced[0];
        int result = ReturnedIntArrayUpdates.replaceRow(replaced);
        require(result == 17 && oldRow[0] == 17 && replaced[0][0] == 100,
                "replace-row", oldRow[0] + ":" + replaced[0][0] + ":" + result);

        int[][] different = { { 2 }, { 8 } };
        ReturnedIntArrayUpdates.different(different, 3);
        require(different[0][0] == 11, "different", Integer.toString(different[0][0]));
    }
}
