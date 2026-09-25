class NestedRunner {
  public static void main(String[] args) throws Exception {
    System.out.println(((Inner) Nested.class.getMethod("child").getDefaultValue()).value());
    System.out.println(((Inner[]) Nested.class.getMethod("children").getDefaultValue()).length);
  }
}
