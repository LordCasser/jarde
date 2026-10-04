package demo;
public enum C3 implements IOps {
    TIMES("*") { @Override public double apply(double x, double y){ return x*y; } },
    DIVIDE("/") { @Override public double apply(double x, double y){ return x/y; } };
    private final String op;
    C3(String op){ this.op=op; }
    public String getT(){ return op; }
}
