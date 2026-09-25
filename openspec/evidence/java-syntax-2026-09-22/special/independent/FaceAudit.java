package probe;
public interface FaceAudit {
 default int read(){return 17;}
 default void touch(int x){ParentAudit.bump(x*2);}
}
