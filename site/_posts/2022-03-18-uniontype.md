---
title: 共用体で重ねられる型
tags: csharp
---

C#では型に`System.Runtime.InteropServices`名前空間の`StructLayout(LayoutKind)`属性をつけることでレイアウトを指定できます．

詳しい解説は[こちら](https://ufcpp.net/study/csharp/interop/memorylayout/)にお譲りして，`LayoutKind.Explicit`を指定したときについて解説します．

参照と値を重ねると
- 値を変えたとき参照が壊れてしまう．(セグメンテーション違反を引き起こす)
- 参照と値の区別ができない部分が発生する．(GCをうまく動かせない)

等々結構面倒な事が起こります．そのため.NETランタイムは参照と値を重ねられません．

どのような型が共用体で重ねられるのか確認しましょう.

- 値のみを含む型(unmanaged型)

```cs
[StructLayout(LayoutKind.Explicit)]
struct A
{
    [FieldOffset(0)]
    public int intValue;
    [FieldOffset(0)]
    public long longValue;
}

var instance = new A();
instance.longValue = 0xFFFFFFFF;
Console.WriteLine(instance.intValue);// -1
```

- 値と参照が重なった型

```cs
[StructLayout(LayoutKind.Explicit)]
struct B
{
    [FieldOffset(0)]
    public string stringValue
    [FieldOffset(0)]
    public nint nintValue;
}

var instance = new B();// TypeLoadException発生
```

- 参照を含む値型が重なった型

```cs
[StructLayout(LayoutKind.Explicit)]
struct C
{
    public struct D
    {
        public string stringValue;
        public int intValue;
    }

    public struct E
    {
        public string stringValue;
        public int intValue;
    }

    [FieldOffset(0)]
    public D dValue;
    [FieldOffset(0)]
    public E eValue;
}

var instance = new C();
instance.dValue.stringValue = "str";
Console.WriteLine(instance.eValue.objectValue);// str
```
意外なことにunmanaged型じゃなくても重ねることが可能みたいです．

- ジェネリック型

```cs
[StructLayout(LayoutKind.Explicit)]
struct F<T>
{
    [FieldOffset(0)]
    public int intValue;
    [FieldOffset(0)]
    public T tValue;
}

var instance = new F<int>();// TypeLoadException
```
やっぱりジェネリック型はだめ．

# 結果
<strong>共用体で重ねられる型は，最終的に参照と値が重ならなければなんでも良い．けどジェネリック型はだめ．</strong>