# Rustの非同期 完全に理解した

ﾅﾝﾓﾜｶﾗﾝ

Rustは非同期の仕組みが言語から提供されていません．その代わりすべて自作することができます．
といってもasync/awaitを使うためのtraitは決まっていて，async関数はFuture traitを実装する型の実体を返す必要があります．

```Rust
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}

pub struct Context<'a> { /* private fields */ }
impl<'a> Context<'a> {
    pub fn waker(&self) -> &'a Waker;
    /* 一部省略 */
}

pub struct Waker { /* private fields */ }
impl Waker {
    pub fn wake(self);
    pub fn wake_by_ref(&self);
    pub fn will_wake(&self, other: &Waker) -> bool;
    /* 一部省略 */
}
/* Pinは省略 */
```

Future traitとこれに関する型はこのように定義されています．

なぜこの様になっているのか考えてみます．
