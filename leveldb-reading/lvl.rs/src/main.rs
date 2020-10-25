mod sstable;

fn main() {
    println!("Hello, world!");
    sstable::dumpSSTable("/home/YuhuiLiu/play/leveldemo/build/test.db/000005.ldb");
}
