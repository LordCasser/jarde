package demo;
public enum C6 implements IOps {
    TIMES("*") { @Override public double apply(double x, double y){ return x*y; } },
    DIVIDE("/") { @Override public double apply(double x, double y){ return x/y; } };
    private final String x;
    C6(String x){ this.x=x; }
    public String getX(){ return x; }
}
