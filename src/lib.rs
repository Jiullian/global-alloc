#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

// Imports obligatoires
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::mem;

extern crate alloc;
use alloc::vec::Vec;

// Structure de données des blocs libres
struct BlocLibre {
    // size : représente la taille du bloc libre en octets
    size: usize,
    // next : pointeur optionnel mutable qui va venir pointer vers le prochain bloc libre
    next: Option<*mut BlocLibre>
}

// Concrètement, on peut s'imaginer ça :
// +-----------+      +-----------+      +-----------+
// | BlocLibre | ---> | BlocLibre | ---> | BlocLibre | ---> None
// +-----------+      +-----------+      +-----------+

// Implémentation de l'allocateur
pub struct AllocateurListeLibre{
    // head : représente le début de la liste chaînée des blocs libres (sera vide ou contiendra un pointeur).
    head : UnsafeCell<Option<*mut BlocLibre>>,
}

impl AllocateurListeLibre{
    // Définission de la méthode new()
    // On crée un Allocateur à liste libre dans lequelle on défini head à None
    pub const fn new() -> Self{
        AllocateurListeLibre{
            head: UnsafeCell::new(None),
        }
    }

    // Initialisation d'une région mémoire libre
    pub unsafe fn initialisation(&self, heap_start: usize, heap_size: usize){
        self.ajout_region_libre(heap_start, heap_size);
    }

    //ajout_region_libre prend 3 paramètres :
    //      - &self : référence à l'instance de l'allocateur
    //      - addr : adresse de départ de la nouvelle région mémoire libre à ajouter
    //      - size : taille de la nouvelle région en question

    unsafe fn ajout_region_libre(&self, addr: usize, size: usize){
        // Alignement de l'adresse et de la taille
        // align : calcule l'alignement recquis pour un BlocLibre
        let align = mem::align_of::<BlocLibre>();
        // aligned_addr : calcule l'adresse alignée suivante en respectant l'alignement requis
        // Supposons que :
        //      - addr = 1003
        //      - align = 8
        // 1003 + 8 - 1 = 1010
        // !(8 - 1) = !7 = 0xFFFFFFF8
        // 1010 & 0xFFFFFFF8 = 1008
        let aligned_addr = (addr + align - 1) & !(align - 1);
        // size : ajustement de la taille
        // Continuons notre exemple :
        //      - size = 500 octets
        // (1008 - 1003) = 5 octets
        // 500 - 5 = 495 octets
        let size = size - (aligned_addr - addr);

        // Vérification de la taille minimale
        if size < mem::size_of::<BlocLibre>(){
            return;
        }

        // Convertit l'adresse alignée en un pointeur vers un bloc libre
        let block = aligned_addr as *mut BlocLibre;
        // Initialisation d'un nouveau bloc libre
        (*block).size = size;
        (*block).next = (*self.head.get()).take();
        // Mise à jour de la liste libre
        (*self.head.get()) = Some(block);
    }

    // Voici un exemple plus visuel de la fonction
    // Avant l'ajout :
    // self.head -> [BlocLibre A] -> [BlocLibre B] -> None
    // Après l'ajout :
    // self.head -> [Nouveau Bloc] -> [BlocLibre A] -> [BlocLibre B] -> None
}

// Implémentation du trait GlobalAlloc pour l'allocateur à liste libre
unsafe impl GlobalAlloc for AllocateurListeLibre{
    // Dans un allocateur nous avons besoin de 2 méthodes :
    // - Alloc
    // Alloc prend 2 paramètres :
    //      &self : Référence à l'instance de l'allocateur
    //      layout : taille et alignement
    // - Dealloc
    // Dealloc prend 3 paramètres :
    //      &self : Référence à l'instance de l'allocateur
    //      ptr : pointeur vers le début du bloc de mémoire à dealloc
    //      layout : taille et alignement


    // Cas de figure d'utilisation d'alloc pour bien comprendre
    // Supposons la liste des blocs suivants :
    // head -> BlocLibre(size=64, next) -> BlocLibre(size=128, next) -> BlocLibre(size=256, None)
    // Demande d'allocation : layout.size() = 100
    // Premier Bloc = 64 < 100      On passe au bloc suivant car la taille est insuffisante
    // Deuxième BLoc 128 > 100      On retire le bloc de la liste, et on retourne le pointeur vers ce bloc
    // Liste des blocs libres mise à jour :
    // head -> BlocLibre(size=64, next) -> BlocLibre(size=256, None)

    // Dealloc est beaucoup plus simple à comprendre
    // La fonction va simplement récupérer le bloc en question
    // puis faire des modifications afin qu'il soit
    // de nouveau considérer comme un bloc libre

    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // current : sert de pointeur pour traverse la liste des blocs libres
        let mut current = self.head.get();

        // Tant que nous avons un bloc dans Option nous retournons au début de la boucle
        while let Some(block) = (*current).as_mut(){
            // Vérifier si le bloc courant est suffisamment grand pour la demande d'allocation
            if(**block).size >= layout.size(){
                // ptr : pointeur vers la mémoire allouée
                let ptr = *block as *mut u8;
                // Retirer le bloc de la liste libre car on est en train de l'allouer
                *current = (**block).next.take();
                return ptr;
            }else{
                // Dans le cas ou le bloc n'est pas suffisamment grand, on avance dans la liste des blocs libres
                current = &mut (**block).next as *mut Option<*mut BlocLibre>;
            }
        }
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout){
        // head : pointeur qui nous permettra de modifier la liste des blocs libres
        let head = self.head.get();
        // block : pointeur vers le bloc de mémoire à désallouer, on va ensuite définir size et next pour le retransformer en bloc libre
        let block = ptr as *mut BlocLibre;
        (*block).size = layout.size();
        (*block).next = (*head).take();
        (*head) = Some(block);
    }
}

// Implémentation de Sync pour l'allocateur
unsafe impl Sync for AllocateurListeLibre{}

// Initialisation de la HEAP, étant donné qu'on est en no_std il faut la définir
#[cfg(feature = "heap1")]
const HEAP_SIZE: usize = 1024 * 1024; // = 1 Mo

#[cfg(feature = "heap2")]
const HEAP_SIZE: usize = 2 * 1024 * 1024; // 2 MB

#[cfg(feature = "heap3")]
const HEAP_SIZE: usize = 4 * 1024 * 1024; // 4 MB

#[cfg(not(any(feature = "heap1", feature = "heap2", feature = "heap3")))]
const HEAP_SIZE: usize = 1024 * 1024; // Default to 1 MB

static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

// Déclaratioon de l'allocateur global
#[global_allocator]
static ALLOCATEUR: AllocateurListeLibre = AllocateurListeLibre::new();

// Fonction d'initialisation de l'allocateur
pub unsafe fn initalisation_allocateur(){
    ALLOCATEUR.initialisation(HEAP.as_ptr() as usize, HEAP_SIZE);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    test();
    loop {}
}

// Fonction de test
pub fn test(){
    unsafe {initalisation_allocateur()};
    let mut vec = Vec::new();
    for i in 0..100{
        vec.push(i);
    }

    // Vérifier que les valeurs sont correctes
    for(i, &val) in vec.iter().enumerate(){
        assert_eq!(i, val);
    }
}

// Gestionnaire d'erreur d'allocation
#[alloc_error_handler]
fn alloc_error(layout: Layout) -> ! {
    panic!("ERREUR ALLOCATION: {:?}", layout);
}