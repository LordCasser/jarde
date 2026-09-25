public final class Run {
  public static void main(String[] args) {
    System.out.println(RefusedCast.fieldCast());
    try { System.out.println(RefusedCast.instanceCast(null)); } catch (Throwable e) { System.out.println(e.getClass().getName()); }
    System.out.println(RefusedCast.chainCast());
  }
}
