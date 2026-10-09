public final class Runner {
    private static void require(boolean condition, String label, String value) {
        if (!condition) {
            throw new AssertionError(label + ":" + value);
        }
        System.out.println("ok:" + label + ":" + value);
    }

    public static void main(String[] args) {
        int[][] two = { { 1, 2 }, { 3, 30 } };
        NestedIntUpdates.plain2(two, 1, 1, 5);
        require(two[1][1] == 35, "plain2", Integer.toString(two[1][1]));

        int[][][] three = { { { 1, 2 }, { 3, 4 } }, { { 5, 6 }, { 7, 8 } } };
        NestedIntUpdates.plain3(three, 1, 0, 1, 8);
        require(three[1][0][1] == 14, "plain3", Integer.toString(three[1][0][1]));

        int[] one = { 9 };
        NestedIntUpdates.scalar(one, 0, 5);
        require(one[0] == 14, "scalar", Integer.toString(one[0]));

        int[][] traced = { { 10 } };
        NestedIntUpdates.trace = 0;
        NestedIntUpdates.traced(traced, 0, 0, 5);
        require(NestedIntUpdates.trace == 123 && traced[0][0] == 15,
                "traced", NestedIntUpdates.trace + ":" + traced[0][0]);

        NestedIntUpdates.trace = 0;
        boolean outerNull = false;
        try {
            NestedIntUpdates.traced(null, 0, 0, 5);
        } catch (NullPointerException expected) {
            outerNull = true;
        }
        require(outerNull && NestedIntUpdates.trace == 1,
                "outer-null", Integer.toString(NestedIntUpdates.trace));

        NestedIntUpdates.trace = 0;
        boolean outerBounds = false;
        try {
            NestedIntUpdates.traced(new int[1][1], 2, 0, 5);
        } catch (ArrayIndexOutOfBoundsException expected) {
            outerBounds = true;
        }
        require(outerBounds && NestedIntUpdates.trace == 1,
                "outer-oob", Integer.toString(NestedIntUpdates.trace));

        NestedIntUpdates.trace = 0;
        boolean nullRow = false;
        try {
            NestedIntUpdates.traced(new int[][] { null }, 0, 0, 5);
        } catch (NullPointerException expected) {
            nullRow = true;
        }
        require(nullRow && NestedIntUpdates.trace == 12,
                "null-row", Integer.toString(NestedIntUpdates.trace));

        NestedIntUpdates.trace = 0;
        boolean innerBounds = false;
        try {
            NestedIntUpdates.traced(new int[][] { { 9 } }, 0, 2, 5);
        } catch (ArrayIndexOutOfBoundsException expected) {
            innerBounds = true;
        }
        require(innerBounds && NestedIntUpdates.trace == 12,
                "inner-oob", Integer.toString(NestedIntUpdates.trace));

        int[][] replaced = { { 10 } };
        int[] oldRow = replaced[0];
        int result = NestedIntUpdates.replaceRow(replaced);
        require(result == 7 && oldRow[0] == 17 && replaced[0][0] == 100,
                "replace-row", oldRow[0] + ":" + replaced[0][0] + ":" + result);

        int[][] different = { { 2 }, { 8 } };
        NestedIntUpdates.different(different, 3);
        require(different[0][0] == 11, "different", Integer.toString(different[0][0]));
    }
}
