package demo;
public enum C2 implements IOps {
    TIMES("*") { @Override public double apply(double x, double y){ return x*y; } },
    DIVIDE("/") { @Override public double apply(double x, double y){ return x/y; } };
    private final String t;
    C2(String t){ this.t=t; }
    public String getT(){ return t; }
}
