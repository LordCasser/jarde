public class OverloadProbe extends OverloadParent {
 public int own(Object x){return 3;}
 public int own(String x){return 4;}
 public int number(int x){return 7;}
 public int number(char x){return 8;}
 public int superObject(){return super.pick((Object)"x");}
 public int ownObject(){return own((Object)"x");}
 public int primitive(char c){return number((int)c);}
}
