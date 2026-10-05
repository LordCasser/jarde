public class NL {
    static final Object LOCK = new Object();
    static class Box { public String toString(){ return Thread.holdsLock(NL.class) ? "Y" : "N"; } }   // 判别：求值时是否持内层锁
    static String probe(Object o){
        synchronized(LOCK){
            synchronized(NL.class){ return "n" + o; }        // 求值必须发生在内层 monitorexit 前
        }
    }
    public static void main(String[] a){ System.out.println(probe(new Box())); }
}
