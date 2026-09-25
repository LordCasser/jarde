public interface FinalInterface {
  Object VALUE = FinalSupport.object();
  static int result() { return FinalSupport.check(VALUE); }
}
