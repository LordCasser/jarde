class BoundaryRunner {
  static String lookup(String name) {
    try {
      return Measure.valueOf(name).name();
    } catch (Throwable failure) {
      return failure.getClass().getSimpleName();
    }
  }

  public static void main(String[] args) {
    Measure[] values = Measure.values();
    StringBuilder order = new StringBuilder();
    for (Measure value : values) {
      if (order.length() != 0) order.append(',');
      order.append(value.name()).append(':').append(value.ordinal()).append(':').append(value.units);
    }
    System.out.println("values=" + order);
    System.out.println("lookup=LOW:" + lookup("LOW") + ",HIGH:" + lookup("HIGH")
        + ",MISSING:" + lookup("MISSING"));
    System.out.println("sum=" + Measure.totalUnits + ":" + Measure.sumUnits());
    System.out.println("helper-calls=" + BoundarySupport.calls);
  }
}
