package cf08nested;

public final class NestedEffectfulNegatives {
  static int calls;
  static int cost(int value) { calls++; return value; }

  public static int extraEntry(int[] xs) {
    int result;
    if (xs == null) {
      result = -1;
    } else {
      int i;
      if (calls == 100) i = 1;
      else i = 0;
      while (true) {
        if (i >= xs.length) { result = cost(7); break; }
        result = xs[i];
        if (result == 3) break;
        i++;
      }
      result = result + calls;
    }
    return result;
  }

  public static int differentTarget(int[] xs) {
    int result;
    if (xs == null) {
      result = -1;
    } else {
      int i = 0;
      while (true) {
        if (i >= xs.length) { result = cost(7); return result + calls; }
        result = xs[i];
        if (result == 3) break;
        i++;
      }
      result = result + calls;
    }
    return result;
  }

  public static int bypassTail(int[] xs) {
    int result;
    outer: {
      if (xs == null) {
        result = -1;
      } else {
        int i = 0;
        while (true) {
          if (i >= xs.length) { result = cost(7); break outer; }
          result = xs[i];
          if (result == 3) break;
          i++;
        }
        result = result + calls;
      }
    }
    return result;
  }

  public static int thirdJoinInput(int[] xs) {
    int result;
    if (xs == null) {
      result = -1;
    } else {
      int i = 0;
      if (calls == 100) {
        result = 2;
      } else {
        while (true) {
          if (i >= xs.length) { result = cost(7); break; }
          result = xs[i];
          if (result == 3) break;
          i++;
        }
      }
      result = result + calls;
    }
    return result;
  }

  public static int withHandler(int[] xs) {
    int result;
    if (xs == null) {
      result = -1;
    } else {
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
      result = result + calls;
    }
    return result;
  }
}
