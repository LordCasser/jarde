package cf03;

public class Runner {
    public static void main(String[] args) {
        for (String value : new String[]{"a", "b", "3", "$", "z"}) {
            int result = BranchShapes.chain(value);
            System.out.println(result + ":" + BranchShapes.hits);
        }
        for (int[] branch : new int[][]{{1, 0, 1}, {1, 1, 1}, {0, 0, 1}, {0, 1, 1}}) {
            BranchShapes.hits = 0;
            boolean result = BranchShapes.nested(branch[0] != 0, branch[1], branch[2]);
            System.out.println(result + ":" + BranchShapes.hits);
        }
        for (String value : new String[]{null, "", "a", "b", "z", "ab"}) {
            System.out.println(BranchShapes.guards(value));
        }
    }
}
