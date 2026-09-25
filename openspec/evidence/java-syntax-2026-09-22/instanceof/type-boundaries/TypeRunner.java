public class TypeRunner {
  public static void main(String[] args) {
    for (String value : new String[] {null, "value"}) {
      System.out.println("class=" + InstanceOfTypes.unrelated(value));
      System.out.println("interface=" + InstanceOfTypes.unrelatedInterface(value));
      System.out.println("related=" + InstanceOfTypes.related(value));
    }
    System.out.println("array-null=" + InstanceOfTypes.unrelatedArray(null));
    System.out.println("array-value=" + InstanceOfTypes.unrelatedArray(new String[0]));
    System.out.println("functional=" + InstanceOfTypes.functional());
    TypeEffects.calls = 0;
    System.out.println("call=" + InstanceOfTypes.called() + ":" + TypeEffects.calls);
    TypeEffects.fail = true;
    TypeEffects.calls = 0;
    try {
      InstanceOfTypes.called();
      System.out.println("producer=returned");
    } catch (RuntimeException error) {
      System.out.println("producer=" + error.getClass().getName() + ":" + TypeEffects.calls);
    }
  }
}
