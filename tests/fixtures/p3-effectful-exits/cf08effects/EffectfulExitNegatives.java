package cf08effects;

public final class EffectfulExitNegatives {
  static int calls;
  static int cost(int value) { calls++; return value; }

  public static int extraCall(int[] xs) {
    int result;
    int i = 0;
    while (true) {
      if (i >= xs.length) { result = cost(7); break; }
      cost(1);
      result = xs[i];
      if (result == 3) break;
      i++;
    }
    return result + calls;
  }

  public static int differentTarget(int[] xs) {
    int result;
    int i = 0;
    while (true) {
      if (i >= xs.length) { result = cost(7); return result + calls; }
      result = xs[i];
      if (result == 3) break;
      i++;
    }
    return result + calls;
  }

  public static int extraLatch(int[] xs) {
    int result;
    int i = 0;
    while (true) {
      if (i >= xs.length) { result = cost(7); break; }
      result = xs[i];
      if (result == 3) break;
      if (result == 2) i += 2;
      else i++;
    }
    return result + calls;
  }

  public static int withHandler(int[] xs) {
    int result;
    int i = 0;
    while (true) {
      if (i >= xs.length) {
        try { result = cost(7); }
        catch (RuntimeException ex) { result = 8; }
        break;
      }
      result = xs[i];
      if (result == 3) break;
      i++;
    }
    return result + calls;
  }
}
