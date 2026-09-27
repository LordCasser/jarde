package cf08nested;
public final class Runner {
  public static void main(String[] args) {
    int[][] cases = { null, new int[0], new int[] { 1, 3 }, new int[] { 1, 2 } };
    for (int[] values : cases) {
      NestedEffectful.calls = 0;
      System.out.println(NestedEffectful.pick(values) + ":" + NestedEffectful.calls);
    }
  }
}
