public class SR {
    static abstract class Builder<T extends Builder<T>> {
        String name; int level;
        abstract T self();
        @SuppressWarnings("unchecked")
        T named(String n){ name = n; return (T) this; }
        T at(int l){ level = l; return self(); }
    }
    static class HttpBuilder extends Builder<HttpBuilder> {
        boolean https = true;
        HttpBuilder secure(boolean s){ https = s; return this; }
        String build(){ return name + ":" + level + ":" + https; }
        HttpBuilder self(){ return this; }
    }
    static String use(){
        return ((HttpBuilder) ((HttpBuilder) new HttpBuilder().named("api")).at(3)).secure(false).build();
    }
    public static void main(String[] a){ System.out.println(use()); }
}
