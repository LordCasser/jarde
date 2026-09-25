class BasicRunner {
  public static void main(String[] args) throws Exception {
    System.out.println(Basic.class.getMethod("count").getDefaultValue());
    System.out.println(Basic.class.getMethod("label").getDefaultValue());
    System.out.println(java.util.Arrays.toString((int[]) Basic.class.getMethod("codes").getDefaultValue()));
  }
}
