package cf08nested;

public final class VerifierRunner {
  public static void main(String[] args) {
    int[] xs = {3};
    NestedEffectfulNegatives.calls = 0;
    System.out.println(NestedEffectfulNegatives.extraEntry(xs));
    NestedEffectfulNegatives.calls = 0;
    System.out.println(NestedEffectfulNegatives.differentTarget(xs));
    NestedEffectfulNegatives.calls = 0;
    System.out.println(NestedEffectfulNegatives.bypassTail(xs));
    NestedEffectfulNegatives.calls = 0;
    System.out.println(NestedEffectfulNegatives.thirdJoinInput(xs));
    NestedEffectfulNegatives.calls = 0;
    System.out.println(NestedEffectfulNegatives.withHandler(xs));
  }
}
