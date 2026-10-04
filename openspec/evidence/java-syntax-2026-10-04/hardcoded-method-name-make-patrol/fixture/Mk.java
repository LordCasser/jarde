public class Mk {
    class In { In(){} }
    In make(){ return new In(); }      // 恰为硬编码名
    In create(){ return new In(); }    // 同形、仅方法名不同
    In newInner(){ return new In(); }  // 同形、另一名字
    In get(){ return new In(); }       // 同形、常见 getter 名
}
