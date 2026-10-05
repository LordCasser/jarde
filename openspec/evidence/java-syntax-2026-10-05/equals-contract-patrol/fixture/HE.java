public class HE {
    private final String name; private final int id; private final double score;   // 手写三字段契约
    HE(String name, int id, double score){ this.name = name; this.id = id; this.score = score; }
    @Override public int hashCode(){
        int result = name != null ? name.hashCode() : 0;      // 经典 31 素数累积
        result = 31 * result + id;
        long temp = Double.doubleToLongBits(score); result = 31 * result + (int) (temp ^ (temp >>> 32));   // double 位形
        return result;
    }
    @Override public boolean equals(Object o){
        if(this == o){ return true; }
        if(o == null || getClass() != o.getClass()){ return false; }   // getClass 严格形
        HE that = (HE) o;
        if(id != that.id){ return false; }
        if(Double.compare(score, that.score) != 0){ return false; }
        return name != null ? name.equals(that.name) : that.name == null;
    }
    @Override public String toString(){ return "HE{name='" + name + "', id=" + id + ", score=" + score + '}'; }
    public static void main(String[] a){ HE x = new HE("a", 1, 2.5); HE y = new HE("a", 1, 2.5); System.out.println(""+x.equals(y)+"/"+(x.hashCode()==y.hashCode())+"/"+x.equals(new HE("b",1,2.5))+"/"+x); }
}
