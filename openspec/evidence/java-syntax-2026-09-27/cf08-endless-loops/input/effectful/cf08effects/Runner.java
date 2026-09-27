package cf08effects;
public final class Runner {
  public static void main(String[] args) {
    for (int[] input : new int[][] {{}, {1,3,4}, {1,2}}) {
      EffectfulExits.calls = 0;
      System.out.println(EffectfulExits.pick(input) + ":" + EffectfulExits.calls);
    }
  }
}
