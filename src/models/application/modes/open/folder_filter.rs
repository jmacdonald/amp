use std::collections::BTreeMap;
use std::path::{PathBuf,Path};

fn no_dot(p:&Path)->&Path{
    if p.ends_with("..."){
        return p.parent().expect("Ends with ..., must have parent");
    }
    p
}
fn parent_dot(mut p:&Path)->Option<PathBuf>{
    p = no_dot(p);
    p = p.parent()?;
    let mut res = PathBuf::from(p);
    res.push("...");
    Some(res)
}



//Limit find the longest common path
pub fn search_as_folders<'a,IT>(it:IT)->Option<Vec<PathBuf>>
    where IT:IntoIterator<Item=&'a Path>
{
    let mut it = it.into_iter();
    let mut common = PathBuf::from(it.next()?);
    let mut deeper = BTreeMap::new();
    
    
    deeper.insert(common.clone(),());


    for v in it {
        let mut v = PathBuf::from(v);
        while ! &v.starts_with(&common) {
            common = (&common).parent()?.into();
        }
        if v == common {
            continue
        }
        deeper.insert(v,());
    }

    let mut d2 = BTreeMap::new();
    for (mut item,_) in deeper{
        while no_dot(&item).parent() != Some(&common){
            item = parent_dot(&item)?;
        }
        d2.insert(item,());
    }
    Some(d2.into_iter().map(|(k,_)|k).collect())
}

#[cfg(test)]
mod test{
    use super::*;
    #[test]
    fn folder_filter_test1(){
        let vals = vec![
            PathBuf::from("/hello/world/buddy"),
            PathBuf::from("/hello/venus/buddy"),
            PathBuf::from("/hello/buddy"),
        ];
        let v2:Vec<&Path> = (&vals).into_iter().map(|x|x as &Path).collect();
        let r = search_as_folders(v2).unwrap(); 
        assert_eq!(r,vec![
                  PathBuf::from("/hello/buddy"),
                  PathBuf::from("/hello/venus/..."),
                  PathBuf::from("/hello/world/..."),
                  ]);
    }        

    #[test]
    fn folder_filter_test2(){
        let vals = vec![
            PathBuf::from("/hello/world/buddy"),
            PathBuf::from("/hello/venus/buddy"),
            PathBuf::from("/hello/buddy"),
            PathBuf::from("/group/buddy"),
        ];
        let v2:Vec<&Path> = (&vals).into_iter().map(|x|x as &Path).collect();
        let r = search_as_folders(v2).unwrap(); 
        assert_eq!(r,vec![
                  PathBuf::from("/group/..."),
                  PathBuf::from("/hello/..."),
                  ]);
    }        


}
