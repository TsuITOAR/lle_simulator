
export function boot(){
    console.log(__dirname)
    import('js/index.js')
        .catch(e => console.error('Error importing `index.js`:', e));
    }
    