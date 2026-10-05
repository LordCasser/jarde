import java.io.*;
public class SZ implements Serializable {
    private static final long serialVersionUID = 1L;          // serialVersionUID
    private String name;
    private transient int cache = 7;                          // transient 字段
    SZ(String n){ name = n; }
    private void writeObject(ObjectOutputStream out) throws IOException {   // JVM 回调私有方法
        out.defaultWriteObject();
        out.writeInt(cache);
    }
    private void readObject(ObjectInputStream in) throws IOException, ClassNotFoundException {
        in.defaultReadObject();
        cache = in.readInt();
    }
    private Object writeReplace() throws ObjectStreamException { return this; }   // 替换回调
    public static void main(String[] a) throws Exception {
        SZ s = new SZ("x");
        ByteArrayOutputStream bos = new ByteArrayOutputStream();
        ObjectOutputStream oos = new ObjectOutputStream(bos);
        oos.writeObject(s); oos.close();
        ObjectInputStream ois = new ObjectInputStream(new ByteArrayInputStream(bos.toByteArray()));
        SZ r = (SZ) ois.readObject();
        System.out.println(""+r.name+"/"+r.cache+"/"+(r != s));
    }
}
