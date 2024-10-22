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
}

// Implémentation du trait GlobalAlloc pour l'allocateur à liste libre
unsafe impl Sync for AllocateurListeLibre{
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