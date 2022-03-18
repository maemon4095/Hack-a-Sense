---
title: C#で高級な列挙型を作る．
tags: csharp
---

```
enum Option<T>
{
    None, Some(T value)
}
```

こんな事したいよねって話．でもC#の列挙型はこんなことできない，からどうにかしたい．

一応デフォルトの列挙型でも値を紐づけることは可能で，属性と拡張メソッドを使って下記のようにします．

```cs
enum EnumWithValue
{
    [Value("value")]
    Key
}

static class EnumWithValueExtension
{
    public static string Value()
    {
        return "value";
    }
}
//呼び出し側
Console.WriteLine(EnumWithValue.Value());// value

```

でもこれで紐づけられるのは基本的にコンパイル時に定数になる値だけになるし，ジェネリクスにもできません．

次に候補になるのが構造体で実装を与える案．これは結構一般的な手法で，enumライクに扱える構造体を作ることはあったりなかったり．作るとこんな感じです．

```cs
readonly struct EnumLike
{
    EnumLike(int num)
    {
        this.num = num;
    }

    //普通のenumと同じ感じ
    public static EnumLike Default => new EnumLike(0);

    //8bit分でフラグ，残りで値を持つとか
    public static EnumLike Argument(int flag, int mode)
    {
        return new EnumLike(flag | (mode << 8))
    }

    readonly int num;
}

//呼び出し側
Console.WriteLine(EnumLike.Argument(1, 2));
```

この構造体での実装はなかなか優秀で，enumの軽量さを保ったまま追加の機能を追加できてジェネリクスも可能．でもIEquatbleとか演算子等々，結構手間がかかるのが玉に瑕です．

というわけで値を持つ列挙型は，型の自動生成で作成します．C#には構造体とクラスの両方があって，enumに似せることを考えると構造体ベースが使いやすいです．でも構造体ベースだけだとOCamlみたいに`type 'a tree = Leaf of 'a | Node of 'a * 'a tree * 'a tree`こんな感じで再帰的な構造を作れなかったり，クラスベースだと実装が非常に簡単になります．

<br><br>

## 実装
* * *
SourceGeneratorでマーカーとして`EnumerationAttribute`がついた部分型から自動生成します．
ガワの仕様を具体例として`int`か`string`を持つ`Enum`型から考えます．
```cs
//元コード
[Enumeration]
partial struct Enum
{
    public static partial Enum Key0(int num);
    public static partial Enum Key1(string str);
}

//生成
Enum key0 = Enum.Key0(int num);
Enum key1 = Enum.Key1(string str);

//分解
key0.Key0(out int num);   //== true
key0.Key1(out string str);//== false

key1.Key0(out int num);   //== false
key1.Key1(out string str);//== true
```
使い心地としてはこんな感じで，生成するメソッドから対になる分解のメソッドを自動生成します．

ここでポイントがあって，C#では同じ名前，引数を持つ関数をスタティック，インスタンスで呼び分けることが出来ません．

なので，値を持たないキーに対しては素朴に関数を作ってしまうと，コンパイルエラーになります．
```cs
class A
{
    public static A Void();
    public bool Void();//コンパイルエラー
}
```
この問題は分解のメソッドも静的にすることで解決します．
```cs
class A
{
    public static A Void();
    public static bool Void(A self);
}
//呼び出し側
var instance = A.Void();
A.Void(instance);//== true
```
ですが，このままでは分解が手間なので拡張メソッドを用います．
```cs
class A
{
    public static A Void();
    public static bool Void(A self);
}

static class A_Extension
{
    public static bool Void(this A self) => A.Void(self);
}

//呼び出し側
var instance = A.Void();
instance.Void();//== true
```
~~こうやって呼び分けできるなら初めからさせてくれい~~

というわけで生成と分解をどちらも静的に実装し，拡張メソッドとして分解を呼ぶ様にします．

ガワの仕様が決まったので，次は内部実装について考えます．

<br><br>

## 内部実装
***

内部実装は本質的には共用体と同等で，クラスと構造体で大幅に異なった実装になります．

### クラスベース実装
***
クラスベース実装では抽象クラスを用います．`Encoding`クラスとかと同じ感じで，privateな内部クラスを抽象型として返します．

```cs
//元コード
[Enumeration]
partial class ClassSample
{
    public static partial ClassSample Key0(int num);
    public static partial ClassSample Key1(string str);
}

//生成コード
abstract partial class ClassSample
{
    class __Key0 : ClassSample
    {
        public _Key0(int num)
        {
            this.num = num;
        }
        public readonly int num;
    }

    class __Key1 : ClassSample
    {
        public _Key1(string str)
        {
            this.str = str;
        }
        public readonly string str;
    }

    public static partial ClassSample Key0(int num) => new __Key0(num);
    public static partial ClassSample Key1(string str) => new __Key1(str);

    public static bool Key0(ClassSample self, out int num)
    {
        if(self is __Key0 instance)
        {
            num = instance.num;
            return true;
        }
        num = default;
        return false;
    }

    public static bool Key1(ClassSample self, out string str)
    {
        if(self is __Key1 instance)
        {
            str = instance.str;
            return true;
        }
        str = default;
        return false;
    }
}

static class ClassSample_Extension
{
    public static bool Key0(this ClassSample self, out int num) => ClassSample.Key0(self, out num);
    public static bool Key1(this ClassSample self, out string str) => ClassSample.Key1(self, out str);
}
```
超簡単ですね．

### 構造体ベース実装
***
{{ base_url }}
構造体において値を重ねる場合，C#に元々ある共用体を用います．ですが，[共用体で重ねられる型]({{site.baseurl}}{% link _posts/2022-03-18-uniontype.md %})には制限があります．

なのでうまく重ねてやる必要があります．
- unmanaged型
    - 共用体にまとめる．
- 参照型
    - object型フィールドにまとめる．
- 値型 / ジェネリック型
    - 重ねずに並べる