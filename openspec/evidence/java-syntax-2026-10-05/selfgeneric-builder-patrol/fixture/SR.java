public class SR {
    static abstract class Builder<T extends Builder<T>> {           // 自引用泛型界
        String name;
        int level;
        abstract T self();
        @SuppressWarnings("unchecked")
        T named(String n){ name = n; return (T) this; }             // unchecked 自类型 cast
        T at(int l){ level = l; return self(); }
    }
    static class HttpBuilder extends Builder<HttpBuilder> {         // 具体叶子
        boolean https = true;
        HttpBuilder secure(boolean s){ https = s; return this; }
        String build(){ return name + ":" + level + ":" + https; }
        HttpBuilder self(){ return this; }
    }
    static String use(){
        return new HttpBuilder().named("api").at(3).secure(false).build();   // 全链混合返回类型
    }
    public static void main(String[] a){ System.out.println(use()); }
}
